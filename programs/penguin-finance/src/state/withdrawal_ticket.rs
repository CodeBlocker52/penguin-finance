use anchor_lang::prelude::*;

#[account]
pub struct WithdrawalTicket {
    /// Vault from which withdrawal is requested
    pub vault: Pubkey,
    
    /// User who requested withdrawal
    pub user: Pubkey,
    
    /// Unique ticket ID
    pub ticket_id: u64,
    
    /// Amount of vault tokens burned
    pub vault_tokens_burned: u64,
    
    /// Expected SOL amount to receive
    pub expected_sol_amount: u64,
    
    /// Epoch when withdrawal was requested
    pub request_epoch: u64,
    
    /// Whether withdrawal is ready to claim
    pub ready_to_claim: bool,
    
    /// Whether ticket has been claimed
    pub claimed: bool,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl WithdrawalTicket {
    pub const LEN: usize = 8 +  // discriminator
        32 + // vault
        32 + // user
        8 +  // ticket_id
        8 +  // vault_tokens_burned
        8 +  // expected_sol_amount
        8 +  // request_epoch
        1 +  // ready_to_claim
        1 +  // claimed
        1;   // bump
}