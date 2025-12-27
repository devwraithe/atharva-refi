use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub organization_pubkey: Pubkey,
    #[max_len(50)]
    pub organization_name: String,

    #[max_len(50)]
    pub species_name: String, // e.g Tiger
    #[max_len(50)]
    pub species_id: String, // e.g panthera_tigris

    pub vault: Pubkey,
    pub pool_mint: Pubkey,

    pub total_deposits: u64,
    pub total_shares: u64,

    pub is_active: bool,

    pub pool_bump: u8,
    pub org_vault_bump: u8,
    pub pool_vault_bump: u8,
}
