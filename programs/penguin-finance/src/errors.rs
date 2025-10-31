use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Operator fee exceeds maximum allowed")]
    OperatorFeeTooHigh,

    #[msg("Vault has reached maximum capacity")]
    VaultCapacityReached,

    #[msg("Deposit amount below minimum")]
    DepositTooSmall,

    #[msg("Insufficient vault balance for staking")]
    InsufficientVaultBalance,

    #[msg("Vault name exceeds maximum length")]
    VaultNameTooLong,

    #[msg("Invalid vault authority")]
    InvalidVaultAuthority,

    #[msg("Collateralization ratio below minimum required")]
    InsufficientCollateral,

    #[msg("Position is healthy, cannot liquidate")]
    PositionHealthy,

    #[msg("Arithmetic overflow occurred")]
    ArithmeticOverflow,

    #[msg("Arithmetic underflow occurred")]
    ArithmeticUnderflow,

    #[msg("Division by zero")]
    DivisionByZero,

    #[msg("Withdrawal not ready yet")]
    WithdrawalNotReady,

    #[msg("Invalid withdrawal ticket")]
    InvalidWithdrawalTicket,

    #[msg("Insufficient buffered SOL for withdrawal")]
    InsufficientBufferedSol,

    #[msg("Position not found")]
    PositionNotFound,

    #[msg("Position has outstanding debt")]
    OutstandingDebt,

    #[msg("Invalid collateral amount")]
    InvalidCollateralAmount,

    #[msg("Invalid pSOL amount")]
    InvalidPsolAmount,

    #[msg("Vault is paused")]
    VaultPaused,

    #[msg("Unauthorized action")]
    Unauthorized,

    #[msg("Invalid vault state")]
    InvalidVaultState,

    #[msg("Stake account not found")]
    StakeAccountNotFound,

    #[msg("Invalid stake state")]
    InvalidStakeState,

    #[msg("Rewards already claimed for this epoch")]
    RewardsAlreadyClaimed,

    #[msg("No rewards available")]
    NoRewardsAvailable,
}