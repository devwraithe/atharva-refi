use crate::constants::{MARINADE_PROGRAM_ID, MIN_YIELD_AMOUNT, ORG_VAULT_SEED, ORG_YIELD_BPS};
use crate::errors::ErrorCode;
use crate::events::YieldStreamed;
use crate::marinade::{marinade_liquid_unstake, LiquidUnstakeAccounts};
use crate::states::MarinadeState;
use crate::{
    constants::{POOL_SEED, POOL_VAULT_SEED},
    states::Pool,
};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{Mint, Token, TokenAccount};

/// Stream 20% of yields to organization vault

#[derive(Accounts)]
pub struct Stream<'info> {
    /// Optional signer used for manual overrides
    /// If None, the instruction must be authorized by pool_vault PDA
    #[account(mut)]
    pub authority: Option<Signer<'info>>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
        constraint = pool.is_active @ ErrorCode::PoolNotActive,
    )]
    pub pool: Account<'info, Pool>,

    /// Signs burn CPI and receives SOL from unstake
    /// Must be authority of `get_msol_from`
    /// Used for recieving SOL - `transfer_sol_to`
    /// CHECK: Used as a signer and other things
    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump,
        constraint = (pool_vault.is_signer || (authority.is_some() && authority.as_ref().unwrap().key() == pool.organization_pubkey)) @ ErrorCode::UnauthorizedStream,
    )]
    pub pool_vault: AccountInfo<'info>,

    /// mSOL token account to burn from
    /// Equivalent of `get_msol_from`
    #[account(
        mut,
        constraint = pool_msol_account.owner == pool_vault.key() @ ErrorCode::InvalidMsolAccount,
    )]
    pub pool_msol_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.organization_pubkey,
        seeds = [ORG_VAULT_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.org_vault_bump,
    )]
    pub organization_vault: SystemAccount<'info>,

    /// Marinade state account
    /// CHECK: Verified by Marinade program
    #[account(mut)]
    pub marinade_state: Account<'info, MarinadeState>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// Vault to receive SOL from unstake
    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_msol_leg: Account<'info, TokenAccount>,

    /// Holds mSOL backing SOL
    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub treasury_msol_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Manual check of the program ID
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_finance_program: AccountInfo<'info>,
}
impl<'info> Stream<'info> {
    pub fn process(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp as u64;

        // Calculate yield
        let current_sol_value = self.compute_pool_sol_value()?;
        let total_yield = self.compute_yield(current_sol_value)?;

        require!(total_yield > MIN_YIELD_AMOUNT, ErrorCode::YieldTooSmall);

        let org_yield_sol = self.calculate_org_yield(total_yield)?;
        let msol_to_unstake = self.sol_to_msol(org_yield_sol)?;

        msg!(
            "Streaming {} SOL (20% of {} total yield)",
            org_yield_sol,
            total_yield
        );

        // Execute yield distribution
        self.unstake_msol(msol_to_unstake)?;
        self.transfer_to_org(org_yield_sol)?;
        self.update_checkpoint(current_sol_value, org_yield_sol)?;

        emit!(YieldStreamed {
            pool: self.pool.key(),
            organization: self.pool.organization_pubkey,
            total_yield: total_yield,
            org_amount: org_yield_sol,
            pool_amount: total_yield - org_yield_sol,
            timestamp: current_time,
        });

        Ok(())
    }

    fn compute_pool_sol_value(&self) -> Result<u64> {
        let msol_balance = self.pool_msol_account.amount;
        if msol_balance == 0 {
            return Ok(0);
        }

        let sol_value = (msol_balance as u128)
            .checked_mul(self.marinade_state.total_virtual_staked_lamports() as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(self.marinade_state.msol_supply as u128)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(sol_value).map_err(|_| ErrorCode::MathError.into())
    }

    fn compute_yield(&self, current_value: u64) -> Result<u64> {
        Ok(current_value.saturating_sub(self.pool.last_streamed_vault_sol))
    }

    fn calculate_org_yield(&self, total_yield: u64) -> Result<u64> {
        let org_amount = (total_yield as u128)
            .checked_mul(ORG_YIELD_BPS)
            .ok_or(ErrorCode::MathError)?
            .checked_div(10_000)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(org_amount).map_err(|_| ErrorCode::MathError.into())
    }

    fn sol_to_msol(&self, sol_amount: u64) -> Result<u64> {
        let msol_amount = (sol_amount as u128)
            .checked_mul(self.marinade_state.msol_supply as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(self.marinade_state.total_virtual_staked_lamports() as u128)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(msol_amount).map_err(|_| ErrorCode::MathError.into())
    }

    fn unstake_msol(&self, msol_amount: u64) -> Result<()> {
        let pool_key = self.pool.key();
        let vault_seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool_key.as_ref(),
            self.pool.organization_pubkey.as_ref(),
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        marinade_liquid_unstake(
            msol_amount,
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
        )?;

        Ok(())
    }

    fn transfer_to_org(&self, amount: u64) -> Result<()> {
        let seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            self.pool.organization_pubkey.as_ref(),
            self.pool.species_id.as_bytes(),
            &[self.pool.pool_vault_bump],
        ];
        let signer = &[&seeds[..]];

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.pool_vault.to_account_info(),
                    to: self.organization_vault.to_account_info(),
                },
                signer,
            ),
            amount,
        )
    }

    fn update_checkpoint(&mut self, current_value: u64, org_cut: u64) -> Result<()> {
        self.pool.last_streamed_vault_sol = current_value
            .checked_sub(org_cut)
            .ok_or(ErrorCode::MathError)?;
        self.pool.last_stream_ts = Clock::get()?.unix_timestamp as u64;
        Ok(())
    }
}
