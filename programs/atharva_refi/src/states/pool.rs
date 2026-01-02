use anchor_lang::prelude::*;

use crate::utilities::bytes_to_string;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub organization_pubkey: Pubkey,
    #[max_len(32)]
    pub organization_name: String,
    /// Store as percentage (20 = 20%)
    /// Converted to basis points (bps) in calculations
    pub organization_yield_bps: u8,

    #[max_len(32)]
    pub species_name: String,
    #[max_len(32)]
    pub species_id: String,
    pub new_species_id: [u8; 32],

    pub vault: Pubkey,
    pub pool_mint: Pubkey,

    pub last_streamed_vault_sol: u64,
    pub last_stream_ts: u64,

    pub total_deposits: u64,
    pub total_shares: u64,

    pub is_active: bool,
    pub is_crank_scheduled: bool, // Track if crank is active

    pub pool_bump: u8,
    pub org_vault_bump: u8,
    pub pool_vault_bump: u8,
}
impl Pool {
    // Helper methods to get strings
    pub fn species_id_str(&self) -> String {
        bytes_to_string(&self.new_species_id)
    }
}
