use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(fee_basis_points: u16, max_capacity: u64, vault_name: String)]
pub struct CreateVault<'info> {
    #[account(
        mut,
        seeds = [FACTORY_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,

    #[account(
        init,
        payer = operator,
        space = Vault::LEN,
        seeds = [VAULT_SEED, factory.key().as_ref(), &factory.vault_count.to_le_bytes()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = operator,
        mint::decimals = 9,
        mint::authority = vault,
        seeds = [b"vault_token_mint", vault.key().as_ref()],
        bump
    )]
    pub vault_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub operator: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateVault>,
    fee_basis_points: u16,
    max_capacity: u64,
    vault_name: String,
) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);
    require!(
        fee_basis_points <= MAX_OPERATOR_FEE_BPS,
        ErrorCode::OperatorFeeTooHigh
    );
    require!(
        vault_name.len() <= MAX_VAULT_NAME_LENGTH,
        ErrorCode::VaultNameTooLong
    );

    let factory = &mut ctx.accounts.factory;
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;
    let vault_id = factory.vault_count;

    // Initialize vault
    vault.factory = factory.key();
    vault.vault_id = vault_id;
    vault.operator = ctx.accounts.operator.key();
    vault.vault_token_mint = ctx.accounts.vault_token_mint.key();
    vault.fee_basis_points = fee_basis_points;
    vault.max_capacity = max_capacity;
    vault.total_staked = 0;
    vault.buffered_sol = 0;
    vault.total_shares = 0;
    vault.total_assets = 0;
    vault.last_reward_epoch = clock.epoch;
    vault.accepting_deposits = true;
    vault.vault_name = vault_name.clone();
    vault.active_validators = 0;
    vault.lifetime_rewards = 0;
    vault.bump = ctx.bumps.vault;

    // Increment vault count
    factory.vault_count = factory
        .vault_count
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(VaultCreated {
        vault: vault.key(),
        vault_id,
        operator: ctx.accounts.operator.key(),
        fee_basis_points,
        max_capacity,
        vault_name,
        timestamp: clock.unix_timestamp,
    });

    msg!("Vault {} created by {}", vault_id, ctx.accounts.operator.key());
    msg!("Fee: {}%, Max capacity: {} SOL", fee_basis_points as f64 / 100.0, max_capacity as f64 / 1e9);

    Ok(())
}