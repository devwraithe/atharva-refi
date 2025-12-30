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
    #[msg("Amount is too small")]
    AmountTooSmall,
    #[msg("Unmatched Program Key")]
    MarinadeProgramError,
    #[msg("String too long")]
    StringTooLong,
    #[msg("Invalid input")]
    InvalidInput,
    #[msg("Settlement can only occur once per day")]
    SettlementTooFrequent,

    #[msg("Yield amount too small to settle")]
    YieldTooSmall,

    #[msg("Invalid yield percentage")]
    InvalidYieldPercentage,

    #[msg("Invalid Marinade state account")]
    InvalidMarinadeState,

    #[msg("Invalid mSOL account")]
    InvalidMsolAccount,

    #[msg("Invalid mSOL mint")]
    InvalidMsolMint,

    #[msg("Invalid Magic Program")]
    InvalidMagicProgram,

    #[msg("Execution interval too short (minimum 1 day)")]
    IntervalTooShort,

    #[msg("Invalid iterations count")]
    InvalidIterations,

    #[msg("Serialization error")]
    SerializationError,

    #[msg("Insufficient shares to withdraw")]
    InsufficientShares,

    #[msg("Pool is empty")]
    PoolEmpty,

    #[msg("Invalid token account")]
    InvalidTokenAccount,
}
