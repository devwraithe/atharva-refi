use anchor_lang::prelude::*;
use solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::solana_program::*;

pub const ADMIN_PUBKEY: Pubkey = pubkey!("11111111111111111111111111111111");
pub const POOL_SEED: &str = "pool";
pub const ORG_VAULT_SEED: &str = "org_vault";
pub const POOL_VAULT_SEED: &str = "pool_vault";

// Marinade Finance Program ID
pub const MARINADE_FINANCE: Pubkey = pubkey!("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD");

// Key Marinade accounts (mainnet)
pub const MARINADE_STATE: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
pub const MSOL_MINT: Pubkey = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");

// Marinade Finance
pub const MARINADE_FINANCE_PROGRAM: &str = "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD"; // Mainnet Program ID

///Base % cut for the partner
pub const DEFAULT_BASE_FEE_POINTS: u32 = 1_000; // 10%

///Max % cut for the partner
pub const DEFAULT_MAX_FEE_POINTS: u32 = 10_000; // 100%

pub const DEFAULT_OPERATION_FEE_POINTS: u8 = 0; // 0%
pub const MAX_OPERATION_FEE_POINTS: u8 = 50; // 0.5%

pub const DEFAULT_MAX_NET_STAKE: u64 = 1_000_000_u64
    .checked_mul(LAMPORTS_PER_SOL)
    .expect("Overflow calculating constant");
