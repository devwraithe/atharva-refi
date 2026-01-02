use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub organization_pubkey: Pubkey,
    #[max_len(50)]
    pub organization_name: String,

    /// Store as percentage (20 = 20%)
    /// Converted to basis points (bps) in calculations
    pub organization_yield_bps: u8,

    #[max_len(50)]
    pub species_name: String,
    #[max_len(50)]
    pub species_id: String,

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
