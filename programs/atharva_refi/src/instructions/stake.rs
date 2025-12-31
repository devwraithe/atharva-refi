use crate::constants::{POOL_SEED, POOL_VAULT_SEED};
use crate::marinade::{marinade_liquid_stake, LiquidStakeAccounts};
use crate::states::Pool;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

// Stake supporter deposits from pool to marinade

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    /// Marinade state account
    /// CHECK: Verified by Marinade program
    #[account(mut)]
    pub marinade_state: AccountInfo<'info>,

    #[account(mut)]
    pub msol_mint: Account<'info, Mint>,

    /// Vault to receive deposited SOL from the pool
    /// CHECK: PDA derived and validated by Marinade
    #[account(mut)]
    pub liq_pool_sol_leg: AccountInfo<'info>,

    /// CHECK: Owned and validated by Marinade
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,

    /// Signs token operations during CPI.
    /// CHECK: PDA signer validated by Marinade.
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,

    /// Holds long-term staked SOL backing mSOL supply
    /// CHECK: PDA derived and validated by Marinade
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,

    /// SOL comes out of here
    /// Equivalent to Marinade's `transfer_from`
    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.organization_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump,
    )]
    pub pool_vault: Signer<'info>,

    /// mSOL goes here
    /// Equivalent to Marinade's `mint_to`
    #[account(
        mut,
        token::mint = msol_mint,
        token::authority = pool_vault,
    )]
    pub pool_msol_account: Account<'info, TokenAccount>,

    /// Signs the mint CPI that issues mSOL.
    /// CHECK: PDA signer validated by Marinade.
    pub msol_mint_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}
impl<'info> Stake<'info> {
    /// Stake SOL with Marinade to receive mSOL
    /// Manually constructs the Marinade deposit instruction
    pub fn process(&self, amount: u64) -> Result<()> {
        msg!("Staking SOL on Marinade...");

        let pool_key = self.pool.key();
        let vault_seeds = &[
            POOL_VAULT_SEED.as_bytes(),
            pool_key.as_ref(),
            self.pool.organization_pubkey.as_ref(),
            &[self.pool.pool_vault_bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        marinade_liquid_stake(
            amount,
            LiquidStakeAccounts {
                marinade_state: self.marinade_state.to_account_info(),
                msol_mint: self.msol_mint.to_account_info(),
                liq_pool_sol_leg: self.liq_pool_sol_leg.to_account_info(),
                liq_pool_msol_leg: self.liq_pool_msol_leg.to_account_info(),
                liq_pool_msol_leg_authority: self.liq_pool_msol_leg_authority.to_account_info(),
                reserve_pda: self.reserve_pda.to_account_info(),
                transfer_from: self.pool_vault.to_account_info(),
                mint_to: self.pool_msol_account.to_account_info(),
                msol_mint_authority: self.msol_mint_authority.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
            Some(signer_seeds),
        )?;

        msg!("Successfully staked {} lamports for mSOL", amount);

        Ok(())
    }
}
