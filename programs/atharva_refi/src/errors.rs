use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the designated admin can create pools.")]
    Unauthorized,
}
