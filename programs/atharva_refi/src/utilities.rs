use anchor_lang::{prelude::*, solana_program::hash};

use crate::constants::POOL_VAULT_SEED;

pub fn calculate_ix_discriminator(ix_name: &str) -> Vec<u8> {
    // "global:" prefix for instructions
    let input = format!("global:{}", ix_name);

    // Hash the string using SHA256
    let hash_result = hash::hash(input.as_bytes());

    hash_result.to_bytes()[..8].to_vec()
}
