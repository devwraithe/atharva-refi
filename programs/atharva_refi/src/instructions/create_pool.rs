use anchor_lang::prelude::*;
use crate::events::PoolCreated;
use crate::states::{Pool, PoolStatus};
use crate::constants::{ADMIN_PUBKEY, POOL_SEED, ORG_VAULT_SEED, POOL_VAULT_SEED};
use crate::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(org_pubkey: Pubkey, species_id: String)]
pub struct CreatePool<'info> {
    #[account(
        mut, 
        address = ADMIN_PUBKEY @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [POOL_SEED.as_bytes(), org_pubkey.as_ref(), species_id.as_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), org_pubkey.as_ref()],
        bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [ORG_VAULT_SEED.as_bytes(), org_pubkey.as_ref()],
        bump,
    )]
    pub org_vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreatePool>, 
    org_name: String, 
    org_pubkey: Pubkey, 
    species_name: String,
    species_id: String
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.org_pubkey = org_pubkey;
    pool.org_name = org_name;
    pool.species_name = species_name.clone();
    pool.species_id = species_id.clone();
    pool.status = PoolStatus::Active;
    pool.total_funded = 0;
    pool.pool_bump = ctx.bumps.pool;
    pool.org_vault_bump = ctx.bumps.org_vault;
    pool.pool_vault_bump = ctx.bumps.pool_vault;

    emit!(PoolCreated { org_pubkey, species_name, species_id });

    Ok(())
}