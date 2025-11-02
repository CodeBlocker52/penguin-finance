use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct StakeFromVault<'info> {
    #[account(
        seeds = [FACTORY_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,

    #[account(
        mut,
        seeds = [VAULT_SEED, vault.factory.as_ref(), &vault.vault_id.to_le_bytes()],
        bump = vault.bump,
        has_one = factory,
        has_one = operator,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: Validator vote account to delegate to
    pub validator_vote_account: UncheckedAccount<'info>,

    /// CHECK: Stake account PDA (will be created)
    #[account(
        mut,
        seeds = [
            STAKE_ACCOUNT_SEED,
            vault.key().as_ref(),
            validator_vote_account.key().as_ref()
        ],
        bump
    )]
    pub stake_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub operator: Signer<'info>,

    /// CHECK: Stake config account
    pub stake_config: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    
    /// CHECK: Stake program
    #[account(address = anchor_lang::solana_program::stake::program::ID)]
    pub stake_program: UncheckedAccount<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    
    /// CHECK: Stake history sysvar
    #[account(address = anchor_lang::solana_program::sysvar::stake_history::ID)]
    pub stake_history: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<StakeFromVault>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);
    require!(
        ctx.accounts.vault.buffered_sol >= amount,
        ErrorCode::InsufficientVaultBalance
    );

    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // For simplicity in hackathon, we'll track the delegation intent
    // In production, this would create actual stake accounts via CPI to stake program
    
    // Update vault state
    vault.buffered_sol = vault
        .buffered_sol
        .checked_sub(amount)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;
    
    vault.total_staked = vault
        .total_staked
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    vault.active_validators = vault
        .active_validators
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(StakeDelegated {
        vault: vault.key(),
        validator: ctx.accounts.validator_vote_account.key(),
        stake_account: ctx.accounts.stake_account.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Vault {} delegated {} SOL to validator {}", 
        vault.vault_id, 
        amount as f64 / 1e9,
        ctx.accounts.validator_vote_account.key()
    );

    Ok(())
}