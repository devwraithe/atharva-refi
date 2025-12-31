use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey;

// Atharva ReFi
pub const ADMIN_PUBKEY: Pubkey = pubkey!("11111111111111111111111111111111");
pub const POOL_SEED: &str = "pool";
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_MINT_SEED: &str = "pool_mint";
pub const ORG_VAULT_SEED: &str = "organization_vault";
pub const STREAM_INTERVAL: u64 = 172_800; // 2 days in seconds
pub const STREAM_INTERVAL_MS: u64 = 172_800_000; // 2 days in milliseconds
pub const MIN_YIELD_AMOUNT: u64 = 1_000_000; // 0.001 SOL
pub const ORG_YIELD_BPS: u128 = 2_000; // 20%

// // Marinade Finance
pub const MARINADE_PROGRAM_ID: Pubkey = pubkey!("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD");

// pub mod marinade_finance {
//     use anchor_lang::prelude::*;

//     pub const PROGRAM_ID: Pubkey = pubkey!("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD");
//     pub const STATE: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
//     pub const MSOL_MINT: Pubkey = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
//     pub const MSOL_MINT_AUTH: Pubkey = pubkey!("3JLPCS1qM2zRw3Dp6V4hZnYHd4toMNPkNesXdX9tg6KM");
//     pub const LIQ_POOL_SOL_LEG: Pubkey = pubkey!("UefNb6z6yvArqe4cJHTXCqStRsKmWhGxnZzuHbikP5Q");
//     pub const LIQ_POOL_MSOL_LEG: Pubkey = pubkey!("7GgPYjS5Dza89wV6FpZ23kUJRG5vbQ1GM25ezspYFSoE");
//     pub const TREASURY_MSOL: Pubkey = pubkey!("8ZUcztoAEhpAeC2ixWewJKQJsSUGYSGPVAjkhDJYf5Gd");
//     pub const RESERVE_PDA: Pubkey = pubkey!("Du3Ysj1wKbxPKkuPPnvzQLQh8oMSVifs3jGZjJWXFmHN");
//     pub const MSOL_LEG_AUTH: Pubkey = pubkey!("EyaSjUtSgo9aRD1f8LWXwdvkpDTmXAW54yoSHZRF14WL");
// }
