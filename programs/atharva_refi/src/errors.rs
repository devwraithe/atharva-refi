use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the designated admin can create pools.")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    MathError,
    #[msg("Pool is currently not active")]
    PoolNotActive,
    #[msg("Not enough SOL to deposit")]
    InsufficientFunds,
    #[msg("Invalid amount")]
    InvalidAmount,
}
