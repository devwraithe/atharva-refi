use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::delegate, cpi::DelegateConfig};

use crate::{constants::POOL_SEED, states::Pool};

#[delegate]
#[derive(Accounts)]
pub struct DelegatePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: The pda to delegate
    #[account(
        mut,
        del,
        seeds = [
            POOL_SEED.as_bytes(),
            pool.organization_pubkey.as_ref(),
            &pool.new_species_id
        ],
        bump = pool.pool_bump,
    )]
    pub pool: Account<'info, Pool>,
}
pub fn delegate_process(ctx: Context<DelegatePool>) -> Result<()> {
    let delegate_config = DelegateConfig {
        validator: ctx.remaining_accounts.first().map(|acc| acc.key()),
        ..Default::default()
    };

    let pool = &ctx.accounts.pool;
    let seeds = &[
        POOL_SEED.as_bytes(),
        pool.organization_pubkey.as_ref(),
        &pool.new_species_id,
    ];

    ctx.accounts
        .delegate_pool(&ctx.accounts.payer, seeds, delegate_config)?;

    msg!("Pool account delegated!");

    Ok(())
}
