use anchor_lang::prelude::*;

#[event]
pub struct FactoryInitialized {
    pub factory: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub vault_id: u64,
    pub operator: Pubkey,
    pub fee_basis_points: u16,
    pub max_capacity: u64,
    pub vault_name: String,
    pub timestamp: i64,
}

#[event]
pub struct DepositMade {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub sol_amount: u64,
    pub vault_tokens_minted: u64,
    pub exchange_rate: u64, // Scaled by 1e9
    pub timestamp: i64,
}

#[event]
pub struct StakeDelegated {
    pub vault: Pubkey,
    pub validator: Pubkey,
    pub stake_account: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsDistributed {
    pub vault: Pubkey,
    pub epoch: u64,
    pub total_rewards: u64,
    pub protocol_fee: u64,
    pub operator_fee: u64,
    pub staker_rewards: u64,
    pub new_exchange_rate: u64, // Scaled by 1e9
    pub timestamp: i64,
}

#[event]
pub struct PsolMinted {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub collateral_amount: u64,
    pub psol_minted: u64,
    pub collateral_ratio: u64, // Basis points
    pub timestamp: i64,
}

#[event]
pub struct PsolBurned {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub psol_burned: u64,
    pub collateral_released: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalRequested {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub ticket_id: u64,
    pub vault_tokens_burned: u64,
    pub estimated_sol: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalCompleted {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub ticket_id: u64,
    pub sol_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct PositionLiquidated {
    pub liquidator: Pubkey,
    pub position_owner: Pubkey,
    pub vault: Pubkey,
    pub collateral_seized: u64,
    pub debt_repaid: u64,
    pub liquidation_bonus: u64,
    pub timestamp: i64,
}

#[event]
pub struct VaultBalanceUpdated {
    pub vault: Pubkey,
    pub old_total_assets: u64,
    pub new_total_assets: u64,
    pub rewards_earned: u64,
    pub timestamp: i64,
}