use crate::constants::{POOL_SEED, POOL_VAULT_SEED};
use crate::errors::ErrorCode;
use crate::events::SupporterDeposited;
use crate::states::{Pool, PoolStatus};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub supporter: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), pool.org_pubkey.as_ref(), pool.species_id.as_bytes()],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), pool.org_pubkey.as_ref()],
        bump = pool.org_vault_bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let supporter = &ctx.accounts.supporter;
    let pool = &mut ctx.accounts.pool;
    let pool_vault = &ctx.accounts.pool_vault;

    // Checks
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(pool.status == PoolStatus::Active, ErrorCode::PoolNotActive);
    require!(supporter.lamports() >= amount, ErrorCode::InsufficientFunds);

    // Update pool state
    pool.total_funded = pool
        .total_funded
        .checked_add(amount)
        .ok_or(ErrorCode::MathError)?;

    // Transfer from supporter to pool vault
    let cpi_accounts = Transfer {
        from: supporter.to_account_info(),
        to: pool_vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);

    system_program::transfer(cpi_ctx, amount)?;

    emit!(SupporterDeposited {
        org_pubkey: pool.org_pubkey,
        species_name: pool.species_name.clone(),
        amount,
    });

    Ok(())
}
