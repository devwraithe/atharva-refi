use crate::constants::{MARINADE_PROGRAM_ID, MIN_YIELD_AMOUNT, ORG_VAULT_SEED, ORG_YIELD_BPS};
use crate::errors::ErrorCode;
use crate::events::YieldStreamed;
use crate::marinade::{marinade_liquid_unstake, LiquidUnstakeAccounts};
use crate::{
    constants::{POOL_SEED, POOL_VAULT_SEED},
    states::Pool,
};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Stream<'info> {
    // #[account(mut)]
    // pub auth: Signer<'info>,
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [
            ORG_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.org_vault_bump,
    )]
    pub organization_vault: SystemAccount<'info>,

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
            "Streaming {} mSOL (≈{} SOL, 20% of {} total yield)",
            msol_to_unstake,
            org_yield_sol,
            total_yield
        );

        // Get pool vault balance before unstaking
        let vault_balance_before = self.pool_vault.lamports();

        // Execute yield distribution
        self.unstake_msol(msol_to_unstake)?;

        // Get pool vault balance after unstaking to see actual SOL received
        let vault_balance_after = self.pool_vault.lamports();
        let actual_sol_received = vault_balance_after.saturating_sub(vault_balance_before);

        msg!(
            "Actually received {} SOL from unstaking",
            actual_sol_received
        );

        // Transfer the ACTUAL amount received, not the calculated amount
        self.transfer_to_org(actual_sol_received)?;
        self.update_checkpoint(current_sol_value, actual_sol_received)?;

        emit!(YieldStreamed {
            pool: self.pool.key(),
            organization: self.pool.organization_pubkey,
            total_yield,
            org_amount: actual_sol_received,
            pool_amount: total_yield.saturating_sub(actual_sol_received),
            timestamp: current_time,
        });

        Ok(())
    }

    fn compute_pool_sol_value(&self) -> Result<u64> {
        let msol_balance = self.pool_msol_account.amount;
        if msol_balance == 0 {
            return Ok(0);
        }

        let data = self.marinade_state.try_borrow_data()?;
        let msol_supply = u64::from_le_bytes(data[368..376].try_into().unwrap());
        let total_virtual_staked = u64::from_le_bytes(data[376..384].try_into().unwrap());

        let sol_value = (msol_balance as u128)
            .checked_mul(total_virtual_staked as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(msol_supply as u128)
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
        let data = self.marinade_state.try_borrow_data()?;
        let msol_supply = u64::from_le_bytes(data[368..376].try_into().unwrap());
        let total_virtual_staked = u64::from_le_bytes(data[376..384].try_into().unwrap());

        let msol_amount = (sol_amount as u128)
            .checked_mul(msol_supply as u128)
            .ok_or(ErrorCode::MathError)?
            .checked_div(total_virtual_staked as u128)
            .ok_or(ErrorCode::MathError)?;

        u64::try_from(msol_amount).map_err(|_| ErrorCode::MathError.into())
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
            &self.pool.new_species_id,
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.pool_vault.to_account_info(),
                    to: self.organization_vault.to_account_info(),
                },
                signer_seeds,
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
