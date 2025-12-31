use anchor_lang::prelude::*;

#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub organization_pubkey: Pubkey,
    pub organization_name: String,
    pub species_name: String,
    pub species_id: String,
    pub timestamp: u64,
}

#[event]
pub struct SupporterDeposited {
    pub organization_pubkey: Pubkey,
    pub species_name: String,
    pub amount: u64,
}

#[event]
pub struct YieldStreamed {
    pub pool: Pubkey,
    pub organization: Pubkey,
    pub total_yield: u64,
    pub org_amount: u64,
    pub pool_amount: u64,
    pub timestamp: u64,
}

#[event]
pub struct SupporterWithdrew {
    pub supporter: Pubkey,
    pub pool: Pubkey,
    pub share_amount: u64,
    pub msol_amount: u64,
    pub sol_amount: u64,
    pub timestamp: u64,
}
