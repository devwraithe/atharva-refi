use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};

use crate::constants::MARINADE_PROGRAM_ID;
use crate::marinade::marinade_liquid_unstake;
use crate::{
    constants::{POOL_MINT_SEED, POOL_SEED, POOL_VAULT_SEED},
    errors::ErrorCode,
    events::SupporterWithdrew,
    marinade::LiquidUnstakeAccounts,
    states::Pool,
};

/// Allows a supporter to exit the pool by burning their share tokens.
///
/// This instruction calculates the supporter's proportional claim on the pool's
/// current mSOL holdings. Because 20% of yield is periodically streamed to the
/// organization via a separate automated Crank, any mSOL currently in the
/// pool vault represents the original principal plus the 80% "Supporter Share"
/// of accumulated yield.

#[derive(Accounts)]
pub struct SupporterWithdraw<'info> {
    #[account(mut)]
    pub supporter: Signer<'info>,

    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.pool_bump,
        constraint = pool.is_active @ ErrorCode::PoolNotActive,
    )]
    pub pool: Account<'info, Pool>,

    /// Pool token mint (share tokens)
    #[account(
        mut,
        seeds = [
            POOL_MINT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump,
    )]
    pub pool_mint: Account<'info, Mint>,

    /// Supporter's pool token account (their share tokens)
    #[account(
        mut,
        constraint = supporter_pool_token_account.owner == supporter.key() @ ErrorCode::InvalidTokenAccount,
        constraint = supporter_pool_token_account.mint == pool_mint.key() @ ErrorCode::InvalidTokenAccount,
    )]
    pub supporter_pool_token_account: Account<'info, TokenAccount>,

    /// Marinade state account
    /// CHECK: Verified by Marinade program
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// Vault to receive SOL from unstake
    /// CHECK: PDA derived and validated by Marinade
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Owned and validated by Marinade
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// Holds mSOL backing SOL
    /// CHECK: Verified by Marinade program
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,

    /// mSOL token account to burn from
    /// Equivalent of `get_msol_from`
    /// CHECK: Verified by Marinade program
    #[account(mut)]
    pub pool_msol_account: Account<'info, TokenAccount>,

    /// Signs burn CPI and receives SOL from unstake
    /// Must be authority of `get_msol_from`
    /// Used for recieving SOL - `transfer_sol_to`
    /// CHECK: Verified by Marinade program
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.pool_vault_bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Marinade program
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_program: AccountInfo<'info>,
}

impl<'info> SupporterWithdraw<'info> {
    pub fn process(&mut self, share_amount: u64) -> Result<()> {
        // Validation
        require!(share_amount > 0, ErrorCode::InvalidAmount);
        require!(
            self.supporter_pool_token_account.amount >= share_amount,
            ErrorCode::InsufficientShares
        );

        // Calculate how much of the pool's mSOL this share amount represents
        let (msol_to_unstake, sol_estimated) = self.calculate_withdrawal_amounts(share_amount)?;

        // Update State
        self.update_pool_state(share_amount, sol_estimated)?;

        // Unstake mSOL to the Vault
        // Note: Marinade liquid_unstake takes msol_amount, not sol_amount
        self.unstake_msol(msol_to_unstake)?;

        // Transfer the resulting SOL to Supporter
        self.transfer_sol_to_supporter(sol_estimated)?;

        // Burn the Share Tokens
        self.burn_share_tokens(share_amount)?;

        emit!(SupporterWithdrew {
            supporter: self.supporter.key(),
            pool: self.pool.key(),
            share_amount,
            msol_amount: msol_to_unstake,
            sol_amount: sol_estimated,
            timestamp: Clock::get()?.unix_timestamp as u64,
        });

        Ok(())
    }

    fn calculate_withdrawal_amounts(&self, share_amount: u64) -> Result<(u64, u64)> {
        let total_shares = self.pool_mint.supply;
        let total_msol = self.pool_msol_account.amount;

        require!(total_shares > 0, ErrorCode::PoolEmpty);

        let msol_to_unstake = (share_amount as u128)
            .checked_mul(total_msol as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(total_shares as u128)
            .ok_or(ErrorCode::MathError)?;

        let msol_u64 = u64::try_from(msol_to_unstake).map_err(|_| ErrorCode::MathError)?;
        let sol_estimated = self.msol_to_sol(msol_u64)?;

        Ok((msol_u64, sol_estimated))
    }

    fn unstake_msol(&self, msol_amount: u64) -> Result<()> {
        let pool = &self.pool;

        let seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
            &[pool.pool_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        marinade_liquid_unstake(
            msol_amount, // Passing msol_amount directly to Marinade
            LiquidUnstakeAccounts {
                marinade_state: self.marinade_state.to_account_info(),
                msol_mint: self.msol_mint.to_account_info(),
                liq_pool_sol_leg: self.liq_pool_sol_leg.to_account_info(),
                liq_pool_msol_leg: self.liq_pool_msol_leg.to_account_info(),
                treasury_msol_account: self.treasury_msol_account.to_account_info(),
                get_msol_from: self.pool_msol_account.to_account_info(),
                get_msol_from_authority: self.pool_vault.to_account_info(),
                transfer_sol_to: self.pool_vault.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
            Some(signer_seeds),
        )
    }

    fn transfer_sol_to_supporter(&self, _amount: u64) -> Result<()> {
        let pool = &self.pool;
        let seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
            &[pool.pool_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // FIX: Transfer the actual SOL currently in the vault
        // (which is what just arrived from Marinade)
        let amount_to_transfer = self.pool_vault.lamports();

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.pool_vault.to_account_info(),
                    to: self.supporter.to_account_info(),
                },
                signer_seeds,
            ),
            amount_to_transfer,
        )
    }

    fn burn_share_tokens(&self, amount: u64) -> Result<()> {
        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.pool_mint.to_account_info(),
                    from: self.supporter_pool_token_account.to_account_info(),
                    authority: self.supporter.to_account_info(),
                },
            ),
            amount,
        )
    }

    fn msol_to_sol(&self, msol_amount: u64) -> Result<u64> {
        let data = self.marinade_state.try_borrow_data()?;

        // Marinade State Field Offsets (Standard for MarBms... program):
        // msol_supply is at byte 368
        // total_virtual_staked_lamports is at byte 376

        let msol_supply = u64::from_le_bytes(
            data[368..376]
                .try_into()
                .map_err(|_| ErrorCode::MathError)?,
        );
        let total_virtual_staked_lamports = u64::from_le_bytes(
            data[376..384]
                .try_into()
                .map_err(|_| ErrorCode::MathError)?,
        );

        require!(msol_supply > 0, ErrorCode::MathError);

        let sol_value = (msol_amount as u128)
            .checked_mul(total_virtual_staked_lamports as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(msol_supply as u128)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(sol_value).map_err(|_| ErrorCode::MathError.into())
    }

    fn update_pool_state(&mut self, shares_burned: u64, sol_withdrawn: u64) -> Result<()> {
        self.pool.total_deposits = self.pool.total_deposits.saturating_sub(sol_withdrawn);
        self.pool.total_shares = self.pool.total_shares.saturating_sub(shares_burned);
        Ok(())
    }
}
