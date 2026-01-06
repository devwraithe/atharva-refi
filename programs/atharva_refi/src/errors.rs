use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // --- Authorization Errors ---
    #[msg("Unauthorized to create pool. Only the designated admin can.")]
    CreatePoolUnauthorized,
    #[msg("Unauthorized to stake. Only the designated organization or admin can.")]
    StakingUnauthorized,
    #[msg("Instruction must be signed by the Organization or triggered by the MagicBlock Crank")]
    UnauthorizedStream,
    #[msg("The provided authority does not match the organization's public key")]
    InvalidOrganizationAuthority,
    #[msg("Only the organization can withdraw")]
    UnauthorizedOrganization,

    // --- State & Validation Errors ---
    #[msg("Arithmetic overflow")]
    MathError,
    #[msg("Pool is currently not active")]
    PoolNotActive,
    #[msg("Not enough SOL to deposit")]
    InsufficientFunds,
    #[msg("Not enough SOL to withdraw")]
    InsufficientWithdrawFunds,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Amount is too small")]
    AmountTooSmall,
    #[msg("Input string exceeds the maximum allowed length")]
    InvalidStringLength,
    #[msg("Invalid input parameters")]
    InvalidInput,

    // --- Staking & Yield Errors ---
    #[msg("Unmatched Program Key")]
    MarinadeProgramError,
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

    // --- MagicBlock Crank Errors ---
    #[msg("Invalid Magic Program account provided")]
    InvalidMagicProgram,
    #[msg("Execution interval too short (minimum required interval not met)")]
    IntervalTooShort,
    #[msg("Iterations count must be greater than zero")]
    InvalidIterations,
    #[msg("Failed to serialize MagicBlock instruction data")]
    SerializationError,
    #[msg("Crank automation is already scheduled for this pool")]
    CrankAlreadyScheduled,

    // --- Pool Lifecycle Errors ---
    #[msg("Insufficient shares to withdraw requested amount")]
    InsufficientShares,
    #[msg("Pool currently has no liquidity")]
    PoolEmpty,
    #[msg("Invalid token account provided")]
    InvalidTokenAccount,
}
