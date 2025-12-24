use anchor_lang::prelude::*;

#[event]
pub struct PoolCreated {
    pub org_pubkey: Pubkey,
    pub species_name: String,
    pub species_id: String,
}

#[event]
pub struct SupporterDeposited {
    pub org_pubkey: Pubkey,
    pub species_name: String,
    pub amount: u64,
}
