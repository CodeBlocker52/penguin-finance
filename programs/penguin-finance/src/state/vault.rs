use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    /// Reference to factory
    pub factory: Pubkey,
    
    /// Unique vault ID
    pub vault_id: u64,
    
    /// Vault operator/authority
    pub operator: Pubkey,
    
    /// Vault token mint (pVault-X)
    pub vault_token_mint: Pubkey,
    
    /// Operator fee in basis points (e.g., 500 = 5%)
    pub fee_basis_points: u16,
    
    /// Maximum SOL capacity for this vault
    pub max_capacity: u64,
    
    /// Total SOL staked to validators
    pub total_staked: u64,
    
    /// Buffered SOL not yet staked
    pub buffered_sol: u64,
    
    /// Total vault token shares minted
    pub total_shares: u64,
    
    /// Total assets (staked + buffered + rewards)
    pub total_assets: u64,
    
    /// Last epoch rewards were claimed
    pub last_reward_epoch: u64,
    
    /// Whether vault is accepting deposits
    pub accepting_deposits: bool,
    
    /// Vault name
    pub vault_name: String,
    
    /// Number of active validators
    pub active_validators: u16,
    
    /// Total rewards earned historically
    pub lifetime_rewards: u64,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl Vault {
    pub const LEN: usize = 8 +   // discriminator
        32 +  // factory
        8 +   // vault_id
        32 +  // operator
        32 +  // vault_token_mint
        2 +   // fee_basis_points
        8 +   // max_capacity
        8 +   // total_staked
        8 +   // buffered_sol
        8 +   // total_shares
        8 +   // total_assets
        8 +   // last_reward_epoch
        1 +   // accepting_deposits
        4 + 32 + // vault_name (String with max 32 chars)
        2 +   // active_validators
        8 +   // lifetime_rewards
        1;    // bump

    /// Calculate current exchange rate (SOL per vault token)
    /// Returns rate scaled by 1e9 for precision
    pub fn exchange_rate(&self) -> Result<u64> {
        if self.total_shares == 0 {
            return Ok(1_000_000_000); // 1:1 initial rate
        }
        
        self.total_assets
            .checked_mul(1_000_000_000)
            .and_then(|v| v.checked_div(self.total_shares))
            .ok_or(error!(crate::errors::ErrorCode::ArithmeticOverflow))
    }

    /// Calculate shares to mint for a given SOL amount
    pub fn calculate_shares(&self, sol_amount: u64) -> Result<u64> {
        if self.total_shares == 0 {
            // First deposit: 1:1 ratio
            return Ok(sol_amount);
        }

        // shares = amount * total_shares / total_assets
        sol_amount
            .checked_mul(self.total_shares)
            .and_then(|v| v.checked_div(self.total_assets))
            .ok_or(error!(crate::errors::ErrorCode::ArithmeticOverflow))
    }

    /// Calculate SOL value for given shares
    pub fn shares_to_sol(&self, shares: u64) -> Result<u64> {
        if self.total_shares == 0 {
            return Ok(0);
        }

        // sol = shares * total_assets / total_shares
        shares
            .checked_mul(self.total_assets)
            .and_then(|v| v.checked_div(self.total_shares))
            .ok_or(error!(crate::errors::ErrorCode::ArithmeticOverflow))
    }

    /// Check if vault has capacity for additional deposits
    pub fn has_capacity(&self, amount: u64) -> bool {
        self.total_assets
            .checked_add(amount)
            .map_or(false, |new_total| new_total <= self.max_capacity)
    }
}