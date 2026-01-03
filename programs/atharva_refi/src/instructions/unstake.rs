use crate::{
    constants::{MARINADE_PROGRAM_ID, POOL_SEED, POOL_VAULT_SEED},
    marinade::{marinade_liquid_unstake, LiquidUnstakeAccounts},
    states::Pool,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
pub struct Unstake<'info> {
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
    pub pool_msol_account: AccountInfo<'info>,

    /// Signs burn CPI and receives SOL from unstake
    /// Must be authority of `get_msol_from`
    /// Used for recieving SOL - `transfer_sol_to`
    /// CHECK: Verified by Marinade program
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
        ],
        bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Marinade program
    #[account(address = MARINADE_PROGRAM_ID)]
    pub marinade_program: AccountInfo<'info>,
}

impl<'info> Unstake<'info> {
    /// Unstake mSOL to receive SOL
    /// Manually constructs the Marinade liquid_unstake CPI
    pub fn process(&self, msol_amount: u64) -> Result<()> {
        msg!("Unstaking {} mSOL from Marinade...", msol_amount);

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

        msg!("Successfully unstaked {} mSOL for SOL", msol_amount);

        Ok(())
    }
}
