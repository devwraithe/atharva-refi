use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub org_pubkey: Pubkey,
    #[max_len(50)]
    pub org_name: String,
    #[max_len(50)]
    pub species_name: String, // e.g Tiger
    #[max_len(50)]
    pub species_id: String, // e.g panthera_tigris
    pub status: PoolStatus,
    pub total_funded: u64,
    pub pool_bump: u8,
    pub org_vault_bump: u8,
    pub pool_vault_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum PoolStatus {
    Inactive,
    Active,
}
