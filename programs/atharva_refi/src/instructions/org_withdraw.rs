use crate::constants::{ADMIN_PUBKEY, ORG_VAULT_SEED, POOL_SEED};
use crate::errors::ErrorCode;
use crate::states::Pool;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

#[derive(Accounts)]
pub struct OrgWithdraw<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub organization: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.org_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [ORG_VAULT_SEED.as_bytes(), pool.org_pubkey.as_ref()],
        bump = pool.org_vault_bump,
    )]
    pub org_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<OrgWithdraw>, amount: u64) -> Result<()> {
    let organization = &ctx.accounts.organization;
    let pool = &ctx.accounts.pool;
    let org_vault = &ctx.accounts.org_vault;

    // Checks
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(pool.total_funded >= amount, ErrorCode::InsufficientFunds);

    let pool_bind = pool.key();
    let seeds = &[
        ORG_VAULT_SEED.as_bytes(),
        pool_bind.as_ref(),
        &[ctx.accounts.pool.org_vault_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Transfer from pool vault to organization
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: org_vault.to_account_info(),
            to: organization.to_account_info(),
        },
        signer_seeds,
    );

    system_program::transfer(cpi_ctx, amount)
}
