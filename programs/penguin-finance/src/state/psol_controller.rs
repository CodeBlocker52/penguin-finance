use anchor_lang::prelude::*;

#[account]
pub struct PsolController {
    /// Reference to factory
    pub factory: Pubkey,
    
    /// pSOL mint
    pub psol_mint: Pubkey,
    
    /// Total pSOL minted
    pub total_psol_minted: u64,
    
    /// Total collateral value (in lamports)
    pub total_collateral_value: u64,
    
    /// Minimum collateralization ratio (basis points)
    pub min_collateral_ratio: u64,
    
    /// Liquidation threshold (basis points)
    pub liquidation_threshold: u64,
    
    /// Liquidation bonus (basis points)
    pub liquidation_bonus: u64,
    
    /// Number of active positions
    pub active_positions: u64,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl PsolController {
    pub const LEN: usize = 8 +  // discriminator
        32 + // factory
        32 + // psol_mint
        8 +  // total_psol_minted
        8 +  // total_collateral_value
        8 +  // min_collateral_ratio
        8 +  // liquidation_threshold
        8 +  // liquidation_bonus
        8 +  // active_positions
        1;   // bump

    /// Calculate global collateralization ratio
    pub fn collateralization_ratio(&self) -> Result<u64> {
        if self.total_psol_minted == 0 {
            return Ok(u64::MAX);
        }

        // ratio = (collateral_value / psol_minted) * 10000
        self.total_collateral_value
            .checked_mul(10000)
            .and_then(|v| v.checked_div(self.total_psol_minted))
            .ok_or(error!(crate::errors::ErrorCode::ArithmeticOverflow))
    }
}

#[account]
pub struct UserPosition {
    /// User who owns this position
    pub owner: Pubkey,
    
    /// Vault from which collateral originated
    pub vault: Pubkey,
    
    /// pSOL controller
    pub psol_controller: Pubkey,
    
    /// Amount of vault tokens locked as collateral
    pub collateral_amount: u64,
    
    /// Amount of pSOL borrowed/minted
    pub psol_debt: u64,
    
    /// Last epoch position was updated
    pub last_update_epoch: u64,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl UserPosition {
    pub const LEN: usize = 8 +  // discriminator
        32 + // owner
        32 + // vault
        32 + // psol_controller
        8 +  // collateral_amount
        8 +  // psol_debt
        8 +  // last_update_epoch
        1;   // bump

    /// Calculate position's collateralization ratio
    /// vault_exchange_rate: scaled by 1e9
    pub fn collateralization_ratio(&self, vault_exchange_rate: u64) -> Result<u64> {
        if self.psol_debt == 0 {
            return Ok(u64::MAX);
        }

        // collateral_value = collateral_amount * exchange_rate / 1e9
        let collateral_value = self.collateral_amount
            .checked_mul(vault_exchange_rate)
            .and_then(|v| v.checked_div(1_000_000_000))
            .ok_or(error!(crate::errors::ErrorCode::ArithmeticOverflow))?;

        // ratio = (collateral_value / psol_debt) * 10000
        collateral_value
            .checked_mul(10000)
            .and_then(|v| v.checked_div(self.psol_debt))
            .ok_or(error!(crate::errors::ErrorCode::ArithmeticOverflow))
    }

    /// Check if position is healthy
    pub fn is_healthy(&self, vault_exchange_rate: u64, min_ratio: u64) -> Result<bool> {
        let ratio = self.collateralization_ratio(vault_exchange_rate)?;
        Ok(ratio >= min_ratio)
    }

    /// Check if position can be liquidated
    pub fn is_liquidatable(&self, vault_exchange_rate: u64, liquidation_threshold: u64) -> Result<bool> {
        let ratio = self.collateralization_ratio(vault_exchange_rate)?;
        Ok(ratio < liquidation_threshold)
    }
}