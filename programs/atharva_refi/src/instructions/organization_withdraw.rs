use crate::constants::{ORG_VAULT_SEED, POOL_SEED};
use crate::errors::ErrorCode;
use crate::states::Pool;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

#[derive(Accounts)]
pub struct OrganizationWithdraw<'info> {
    #[account(mut)]
    pub organization: Signer<'info>,

    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
        ],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [
            ORG_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
        ],
        bump = pool.org_vault_bump,
    )]
    pub org_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
impl<'info> OrganizationWithdraw<'info> {
    pub fn process(&self, amount: u64) -> Result<()> {
        // Checks
        require!(amount > 0, ErrorCode::InvalidAmount);
        require!(
            self.pool.total_deposits >= amount,
            ErrorCode::InsufficientWithdrawFunds,
        );

        let pool = &self.pool;

        let seeds = &[
            ORG_VAULT_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id,
            &[pool.org_vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Transfer from pool vault to organization
        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            Transfer {
                from: self.org_vault.to_account_info(),
                to: self.organization.to_account_info(),
            },
            signer_seeds,
        );

        system_program::transfer(cpi_ctx, amount)
    }
}
