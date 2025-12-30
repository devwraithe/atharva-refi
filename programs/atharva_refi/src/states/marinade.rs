use anchor_lang::prelude::*;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct MarinadeState {
    pub msol_supply: u64,
    pub total_virtual_staked_lamports: u64,
}
