use crate::actions::{marinade_liquid_unstake, MarinadeLiquidUnstakeAccounts};
use crate::errors::ErrorCode;
use crate::events::YieldStreamed;
use crate::states::MarinadeState;
use crate::{
    constants::{marinade_finance, POOL_SEED, POOL_VAULT_SEED},
    states::Pool,
};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct StreamYields<'info> {
    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), pool.organization_pubkey.as_ref()],
        bump,
    )]
    pub pool_vault: Signer<'info>,

    #[account(mut,
        constraint = pool_msol_account.owner == pool_vault.key() @ ErrorCode::InvalidMsolAccount,
        constraint = pool_msol_account.mint == msol_mint.key() @ ErrorCode::InvalidMsolAccount,
    )]
    pub pool_msol_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        address = pool.organization_pubkey,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.organization_pubkey.as_ref()],
        bump,
    )]
    pub organization_vault: SystemAccount<'info>,

    /// CHECK: Read-only
    #[account(
        address = marinade_finance::STATE @ ErrorCode::InvalidMarinadeState,
    )]
    pub marinade_state: AccountInfo<'info>,

    #[account(
        mut,
        address = marinade_finance::MSOL_MINT @ ErrorCode::InvalidMsolMint,
    )]
    pub msol_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// CHECK: Validated by Marinade program
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,

    /// CHECK: Marinade program
    #[account(address = marinade_finance::PROGRAM_ID)]
    pub marinade_program: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
impl<'info> StreamYields<'info> {
    pub fn process(&mut self) -> Result<()> {
        // Verify checks
        require!(self.pool.is_active, ErrorCode::PoolNotActive);

        let current_time = Clock::get()?.unix_timestamp as u64;
        let time_since_last_settlement = current_time
            .checked_sub(self.pool.last_settlement_ts)
            .ok_or(ErrorCode::MathError)?;

        require!(
            time_since_last_settlement >= 86_400, // 1 day in secs
            ErrorCode::SettlementTooFrequent
        );

        // Computation
        let current_pool_value_sol = self.compute_pool_value_sol()?;

        msg!("Current pool value: {} lamports", current_pool_value_sol);
        msg!(
            "Last settled value: {} lamports",
            self.pool.last_settled_vault_sol
        );

        let yield_amount = self.compute_yield(current_pool_value_sol)?;

        require!(
            yield_amount >= 1_000_000, // 0.001 SOL minimum
            ErrorCode::YieldTooSmall
        );

        msg!("Yield generated: {} lamports", yield_amount);

        let org_yield_amount = self.compute_organization_cut(yield_amount)?;

        msg!(
            "Organization share ({}%): {} lamports",
            self.pool.organization_yield_bps,
            org_yield_amount
        );

        // Realization
        self.realize_organization_yield(org_yield_amount)?;

        // Update state
        self.update_checkpoint(current_pool_value_sol, org_yield_amount)?;

        emit!(YieldStreamed {
            pool: self.pool.key(),
            organization: self.pool.organization_pubkey,
            total_yield: yield_amount,
            org_amount: org_yield_amount,
            pool_amount: yield_amount - org_yield_amount,
            timestamp: current_time,
        });

        Ok(())
    }

    fn compute_pool_value_sol(&self) -> Result<u64> {
        let msol_balance = self.pool_msol_account.amount;

        if msol_balance == 0 {
            return Ok(0);
        }

        let marinade_state = self.deserialize_marinade_state()?;

        let sol_value = (msol_balance as u128)
            .checked_mul(marinade_state.total_virtual_staked_lamports as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(marinade_state.msol_supply as u128)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(sol_value).map_err(|_| ErrorCode::MathError.into())
    }

    fn compute_yield(&self, current_value_sol: u64) -> Result<u64> {
        let last_value = self.pool.last_settled_vault_sol;

        if last_value == 0 {
            return Ok(0);
        }

        let gross_growth = current_value_sol
            .checked_sub(last_value)
            .ok_or(ErrorCode::MathError)?;

        Ok(gross_growth)
    }

    fn compute_organization_cut(&self, yield_amount: u64) -> Result<u64> {
        let bps = (self.pool.organization_yield_bps as u64)
            .checked_mul(100)
            .ok_or(ErrorCode::MathError)?;

        require!(bps <= 10_000, ErrorCode::InvalidYieldPercentage);

        let org_amount = (yield_amount as u128)
            .checked_mul(bps as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(10_000)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(org_amount).map_err(|_| ErrorCode::MathError.into())
    }

    fn realize_organization_yield(&self, sol_amount: u64) -> Result<()> {
        let pool_key = self.pool.key();
        let vault_seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool_key.as_ref(),
            self.pool.organization_pubkey.as_ref(),
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        marinade_liquid_unstake(
            sol_amount,
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
            Some(signer_seeds),
        )?;

        let transfer_accounts = Transfer {
            from: self.pool_vault.to_account_info(),
            to: self.organization_vault.to_account_info(),
        };

        let transfer_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        transfer(transfer_ctx, sol_amount)?;

        Ok(())
    }

    fn update_checkpoint(&mut self, current_value: u64, org_amount: u64) -> Result<()> {
        self.pool.last_settled_vault_sol = current_value
            .checked_sub(org_amount)
            .ok_or(ErrorCode::MathError)?;

        self.pool.last_settlement_ts = Clock::get()?.unix_timestamp as u64;

        msg!(
            "Settlement checkpoint updated: {} lamports at timestamp {}",
            self.pool.last_settled_vault_sol,
            self.pool.last_settlement_ts
        );

        Ok(())
    }

    fn compute_msol_for_sol_amount(&self, sol_amount: u64) -> Result<u64> {
        let marinade_state = self.deserialize_marinade_state()?;

        let msol_amount = (sol_amount as u128)
            .checked_mul(marinade_state.msol_supply as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(marinade_state.total_virtual_staked_lamports as u128)
            .ok_or(ErrorCode::MathError)?;

        let msol_with_buffer = msol_amount
            .checked_mul(101)
            .ok_or(ErrorCode::MathError)?
            .checked_div(100)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(msol_with_buffer).map_err(|_| ErrorCode::MathError.into())
    }

    fn deserialize_marinade_state(&self) -> Result<MarinadeState> {
        let data = self.marinade_state.try_borrow_data()?;

        let state_data = &data[8..];

        MarinadeState::try_from_slice(state_data)
            .map_err(|_| ErrorCode::InvalidMarinadeState.into())
    }
}
