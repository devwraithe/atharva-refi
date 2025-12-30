use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token_interface::{burn, Burn, Mint, TokenAccount, TokenInterface};

use crate::{
    actions::{marinade_liquid_unstake, MarinadeLiquidUnstakeAccounts},
    constants::{marinade_finance, POOL_MINT_SEED, POOL_SEED, POOL_VAULT_SEED},
    errors::ErrorCode,
    events::SupporterWithdrew,
    states::Pool,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub supporter: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
        constraint = pool.is_active @ ErrorCode::PoolNotActive,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), pool.organization_pubkey.as_ref()],
        bump = pool.pool_vault_bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    /// Pool token mint (share tokens)
    #[account(
        mut,
        seeds = [POOL_MINT_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump,
    )]
    pub pool_mint: InterfaceAccount<'info, Mint>,

    /// Supporter's pool token account (their share tokens)
    #[account(
        mut,
        constraint = supporter_pool_token_account.owner == supporter.key() @ ErrorCode::InvalidTokenAccount,
        constraint = supporter_pool_token_account.mint == pool_mint.key() @ ErrorCode::InvalidTokenAccount,
    )]
    pub supporter_pool_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Pool's mSOL account
    #[account(
        mut,
        constraint = pool_msol_account.owner == pool_vault.key() @ ErrorCode::InvalidMsolAccount,
        constraint = pool_msol_account.mint == marinade_finance::MSOL_MINT @ ErrorCode::InvalidMsolAccount,
    )]
    pub pool_msol_account: InterfaceAccount<'info, TokenAccount>,

    /// Marinade accounts for unstaking
    /// CHECK: Marinade state
    #[account(address = marinade_finance::STATE)]
    pub marinade_state: AccountInfo<'info>,

    #[account(mut, address = marinade_finance::MSOL_MINT)]
    pub msol_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Marinade liquidity pool accounts
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,

    /// CHECK: Marinade program
    #[account(address = marinade_finance::PROGRAM_ID)]
    pub marinade_program: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn process(&mut self, share_amount: u64) -> Result<()> {
        // Validation
        require!(share_amount > 0, ErrorCode::InvalidAmount);
        require!(
            self.supporter_pool_token_account.amount >= share_amount,
            ErrorCode::InsufficientShares
        );

        // Calculate supporter's proportion of the pool
        let (msol_to_unstake, sol_amount) = self.calculate_withdrawal_amounts(share_amount)?;

        msg!(
            "Withdrawing {} shares -> {} mSOL -> ~{} SOL",
            share_amount,
            msol_to_unstake,
            sol_amount
        );

        // Unstake mSOL to get SOL
        self.unstake_msol(msol_to_unstake)?;

        // Transfer SOL to supporter
        self.transfer_sol_to_supporter(sol_amount)?;

        // Burn supporter's share tokens
        self.burn_share_tokens(share_amount)?;

        // Update pool state
        self.update_pool_state(share_amount, sol_amount)?;

        emit!(SupporterWithdrew {
            supporter: self.supporter.key(),
            pool: self.pool.key(),
            share_amount,
            msol_amount: msol_to_unstake,
            sol_amount,
            timestamp: Clock::get()?.unix_timestamp as u64,
        });

        Ok(())
    }

    /// Calculates how much mSOL and SOL the supporter gets
    ///
    /// Formula: supporter_msol = (share_amount / total_shares) * total_msol
    /// Then convert mSOL to SOL using Marinade exchange rate
    fn calculate_withdrawal_amounts(&self, share_amount: u64) -> Result<(u64, u64)> {
        let total_shares = self.pool_mint.supply;
        let total_msol = self.pool_msol_account.amount;

        // Edge case: if somehow total shares is 0
        require!(total_shares > 0, ErrorCode::PoolEmpty);

        // Calculate supporter's proportion of mSOL
        // msol_amount = (share_amount * total_msol) / total_shares
        let msol_to_unstake = (share_amount as u128)
            .checked_mul(total_msol as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(total_shares as u128)
            .ok_or(ErrorCode::MathError)?;

        let msol_to_unstake = u64::try_from(msol_to_unstake).map_err(|_| ErrorCode::MathError)?;

        // Convert mSOL to expected SOL amount (for logging/events)
        let sol_amount = self.msol_to_sol(msol_to_unstake)?;

        Ok((msol_to_unstake, sol_amount))
    }

    /// Converts mSOL amount to SOL using Marinade exchange rate
    fn msol_to_sol(&self, msol_amount: u64) -> Result<u64> {
        let data = self.marinade_state.try_borrow_data()?;
        let state = crate::states::MarinadeState::try_from_slice(&data[8..])
            .map_err(|_| ErrorCode::InvalidMarinadeState)?;

        let sol_value = (msol_amount as u128)
            .checked_mul(state.total_virtual_staked_lamports as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(state.msol_supply as u128)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(sol_value).map_err(|_| ErrorCode::MathError.into())
    }

    /// Unstakes mSOL via Marinade liquid unstake
    fn unstake_msol(&self, msol_amount: u64) -> Result<()> {
        let pool_key = self.pool.key();
        let signer_seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool_key.as_ref(),
            self.pool.organization_pubkey.as_ref(),
            &[self.pool.pool_vault_bump],
        ];

        // Use estimated SOL amount (Marinade will give actual based on current rate)
        let estimated_sol = self.msol_to_sol(msol_amount)?;

        marinade_liquid_unstake(
            estimated_sol,
            MarinadeLiquidUnstakeAccounts {
                marinade_state: self.marinade_state.to_account_info(),
                msol_mint: self.msol_mint.to_account_info(),
                liq_pool_sol_leg: self.liq_pool_sol_leg.to_account_info(),
                liq_pool_msol_leg: self.liq_pool_msol_leg.to_account_info(),
                treasury_msol_account: self.treasury_msol_account.to_account_info(),
                burn_msol_from: self.pool_msol_account.to_account_info(),
                burn_msol_authority: self.pool_vault.to_account_info(),
                sol_receiver: self.pool_vault.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
            Some(&[signer_seeds]),
        )
    }

    /// Transfers SOL from pool vault to supporter
    fn transfer_sol_to_supporter(&self, amount: u64) -> Result<()> {
        let pool_key = self.pool.key();
        let seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool_key.as_ref(),
            self.pool.organization_pubkey.as_ref(),
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            Transfer {
                from: self.pool_vault.to_account_info(),
                to: self.supporter.to_account_info(),
            },
            signer_seeds,
        );

        transfer(ctx, amount)
    }

    /// Burns supporter's share tokens
    fn burn_share_tokens(&self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.pool_mint.to_account_info(),
                from: self.supporter_pool_token_account.to_account_info(),
                authority: self.supporter.to_account_info(),
            },
        );

        burn(ctx, amount)
    }

    /// Updates pool state after withdrawal
    fn update_pool_state(&mut self, shares_burned: u64, sol_withdrawn: u64) -> Result<()> {
        // Decrease total deposits by amount withdrawn
        self.pool.total_deposits = self
            .pool
            .total_deposits
            .checked_sub(sol_withdrawn)
            .ok_or(ErrorCode::MathError)?;

        // Decrease total shares
        self.pool.total_shares = self
            .pool
            .total_shares
            .checked_sub(shares_burned)
            .ok_or(ErrorCode::MathError)?;

        Ok(())
    }
}
