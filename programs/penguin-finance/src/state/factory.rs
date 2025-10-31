use anchor_lang::prelude::*;

#[account]
pub struct Factory {
    /// Authority that can update protocol parameters
    pub authority: Pubkey,
    
    /// Protocol treasury for collecting fees
    pub treasury: Pubkey,
    
    /// Total number of vaults created
    pub vault_count: u64,
    
    /// Protocol fee in basis points (e.g., 100 = 1%)
    pub protocol_fee_bps: u16,
    
    /// Whether protocol is paused
    pub paused: bool,
    
    /// pSOL mint address
    pub psol_mint: Pubkey,
    
    /// pSOL controller address
    pub psol_controller: Pubkey,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl Factory {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // treasury
        8 +  // vault_count
        2 +  // protocol_fee_bps
        1 +  // paused
        32 + // psol_mint
        32 + // psol_controller
        1;   // bump
}