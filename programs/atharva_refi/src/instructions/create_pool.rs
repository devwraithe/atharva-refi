use crate::constants::{ADMIN_PUBKEY, ORG_VAULT_SEED, POOL_SEED, POOL_VAULT_SEED};
use crate::errors::ErrorCode;
use crate::events::PoolCreated;
use crate::states::Pool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(organization_pubkey: Pubkey, species_id: String)]
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
        seeds = [POOL_SEED.as_bytes(), organization_pubkey.as_ref(), species_id.as_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), organization_pubkey.as_ref()],
        bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [ORG_VAULT_SEED.as_bytes(), organization_pubkey.as_ref()],
        bump,
    )]
    pub org_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreatePool<'info> {
    pub fn process(
        &mut self,
        organization_name: String,
        organization_pubkey: Pubkey,
        species_name: String,
        species_id: String,
        bumps: &CreatePoolBumps,
    ) -> Result<()> {
        // Verify checks
        require!(organization_name.len() <= 50, ErrorCode::StringTooLong);
        require!(species_name.len() <= 50, ErrorCode::StringTooLong);
        require!(species_id.len() <= 20, ErrorCode::StringTooLong);
        require!(
            !organization_name.is_empty() && !species_name.is_empty() && !species_id.is_empty(),
            ErrorCode::InvalidInput
        );

        // Initialize pool
        let pool = &mut self.pool;

        pool.organization_pubkey = organization_pubkey;
        pool.organization_name = organization_name;
        pool.species_name = species_name.clone();
        pool.species_id = species_id.clone();

        pool.is_active = true;
        pool.total_deposits = 0;
        pool.total_shares = 0;

        pool.pool_bump = bumps.pool;
        pool.org_vault_bump = bumps.org_vault;
        pool.pool_vault_bump = bumps.pool_vault;

        // Emit events
        emit!(PoolCreated {
            organization_pubkey,
            species_name,
            species_id,
        });

        Ok(())
    }
}
