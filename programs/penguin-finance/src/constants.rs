/// Protocol-wide constants

/// Seed for factory PDA
pub const FACTORY_SEED: &[u8] = b"factory";

/// Seed for vault PDA
pub const VAULT_SEED: &[u8] = b"vault";

/// Seed for pSOL controller PDA
pub const PSOL_CONTROLLER_SEED: &[u8] = b"psol_controller";

/// Seed for user position PDA
pub const USER_POSITION_SEED: &[u8] = b"user_position";

/// Seed for withdrawal ticket PDA
pub const WITHDRAWAL_TICKET_SEED: &[u8] = b"withdrawal_ticket";

/// Seed for stake account PDA
pub const STAKE_ACCOUNT_SEED: &[u8] = b"stake_account";

/// Minimum collateralization ratio (110%)
pub const MIN_COLLATERAL_RATIO: u64 = 11000; // Basis points (110%)

/// Liquidation threshold (105%)
pub const LIQUIDATION_THRESHOLD: u64 = 10500; // Basis points (105%)

/// Liquidation bonus for liquidators (5%)
pub const LIQUIDATION_BONUS: u64 = 500; // Basis points (5%)

/// Protocol fee on rewards (1%)
pub const PROTOCOL_FEE_BPS: u16 = 100; // Basis points (1%)

/// Maximum operator fee (15%)
pub const MAX_OPERATOR_FEE_BPS: u16 = 1500; // Basis points (15%)

/// Minimum stake amount (0.1 SOL)
pub const MIN_STAKE_AMOUNT: u64 = 100_000_000; // lamports (0.1 SOL)

/// Maximum vault name length
pub const MAX_VAULT_NAME_LENGTH: usize = 32;

/// Basis points denominator
pub const BASIS_POINTS_DIVISOR: u64 = 10000;

/// Minimum rent-exempt balance
pub const MIN_RENT_EXEMPT: u64 = 1_000_000; // ~0.001 SOL

/// Slots per epoch (approximate, for calculation purposes)
pub const SLOTS_PER_EPOCH: u64 = 432_000;