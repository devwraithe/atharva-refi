use anchor_lang::prelude::*;
use anchor_lang::solana_program::{native_token::LAMPORTS_PER_SOL, pubkey};

pub const ADMIN_PUBKEY: Pubkey = pubkey!("11111111111111111111111111111111");
pub const POOL_SEED: &str = "pool";
pub const POOL_MINT_SEED: &str = "pool_mint";
pub const ORG_VAULT_SEED: &str = "org_vault";
pub const POOL_VAULT_SEED: &str = "pool_vault";

// Marinade Finance Program ID
pub const MARINADE_FINANCE: Pubkey = pubkey!("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD");

pub mod marinade_finance {
    use anchor_lang::prelude::*;

    pub const PROGRAM_ID: Pubkey = pubkey!("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD");
    pub const STATE: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
    pub const MSOL_MINT: Pubkey = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
    pub const LIQ_POOL_SOL_LEG: Pubkey = pubkey!("UefNb6z6yvArqe4cJHTXCqStRsKmWhGxnZzuHbikP5Q");
    pub const LIQ_POOL_MSOL_LEG: Pubkey = pubkey!("7GgPYjS5Dza89wV6FpZ23kUJRG5vbQ1GM25ezspYFSoE");
    pub const TREASURY_MSOL: Pubkey = pubkey!("Du3Ysj1wKbxPKkuPPnvzQLQh8oMSVifs3jGZjJWXFmHN");
}
