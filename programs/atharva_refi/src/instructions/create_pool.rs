use crate::constants::{ADMIN_PUBKEY, ORG_VAULT_SEED, POOL_SEED, POOL_VAULT_SEED};
use crate::errors::ErrorCode;
use crate::events::PoolCreated;
use crate::states::Pool;
use anchor_lang::prelude::*;

/// Creates a conservation pool for a specific species under an organization
///
/// Architecture:
/// - One pool per (organization, species) pair
/// - Each pool has isolated vault for deposits
/// - Organization has isolated vault for yield collection

#[derive(Accounts)]
#[instruction(
    organization_name: String,
    organization_pubkey: Pubkey,
    species_name: String,
    species_id: [u8; 32],
)]
pub struct CreatePool<'info> {
    #[account(
        mut,
        address = ADMIN_PUBKEY @ ErrorCode::CreatePoolUnauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [POOL_SEED.as_bytes(), organization_pubkey.as_ref(), &species_id],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            organization_pubkey.as_ref(),
            &species_id
        ],
        bump,
    )]
    pub pool_vault: SystemAccount<'info>,

    #[account(
        seeds = [
            ORG_VAULT_SEED.as_bytes(),
            organization_pubkey.as_ref(),
            &species_id
        ],
        bump,
    )]
    pub organization_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
impl<'info> CreatePool<'info> {
    pub fn process(
        &mut self,
        organization_name: String,
        organization_pubkey: Pubkey,
        species_name: String,
        species_id: [u8; 32],
        bumps: &CreatePoolBumps,
    ) -> Result<()> {
        // Validation
        require!(species_id[0] != 0, ErrorCode::InvalidStringLength);

        let pool = &mut self.pool;

        pool.organization_pubkey = organization_pubkey;
        pool.organization_name = organization_name.clone();
        pool.organization_yield_bps = 20;
        pool.species_name = species_name.clone();
        pool.species_id = bytes_to_string(&species_id);

        pool.is_active = true;
        pool.is_crank_scheduled = false;
        pool.total_deposits = 0;
        pool.total_shares = 0;
        pool.last_streamed_vault_sol = 0;
        pool.last_stream_ts = 0;

        pool.pool_bump = bumps.pool;
        pool.org_vault_bump = bumps.organization_vault;
        pool.pool_vault_bump = bumps.pool_vault;

        // Convert to strings for event (events can use String)
        let species_id_str = bytes_to_string(&species_id);

        msg!("Seed Org: {:?}", organization_pubkey.as_ref());
        msg!("Seed Species: {:?}", species_id);

        emit!(PoolCreated {
            pool: pool.key(),
            organization_pubkey,
            organization_name,
            species_name,
            species_id: species_id_str,
            timestamp: Clock::get()?.unix_timestamp as u64,
        });

        Ok(())
    }
}

// Helper function to convert fixed byte array to String
fn bytes_to_string(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..len]).to_string()
}
