use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct RequestWithdrawal<'info> {
    #[account(
        seeds = [FACTORY_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,

    #[account(
        mut,
        seeds = [VAULT_SEED, vault.factory.as_ref(), &vault.vault_id.to_le_bytes()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    /// Vault token mint
    pub vault_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        space = WithdrawalTicket::LEN,
        seeds = [
            WITHDRAWAL_TICKET_SEED,
            vault.key().as_ref(),
            user.key().as_ref(),
            &vault.last_reward_epoch.to_le_bytes()
        ],
        bump
    )]
    pub withdrawal_ticket: Account<'info, WithdrawalTicket>,

    /// User's vault token account
    #[account(
        mut,
        associated_token::mint = vault_token_mint,
        associated_token::authority = user,
    )]
    pub user_vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<RequestWithdrawal>, vault_token_amount: u64) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);
    require!(vault_token_amount > 0, ErrorCode::InvalidCollateralAmount);

    let vault = &mut ctx.accounts.vault;
    let withdrawal_ticket = &mut ctx.accounts.withdrawal_ticket;
    let clock = Clock::get()?;

    // Calculate expected SOL
    let expected_sol = vault.shares_to_sol(vault_token_amount)?;

    // Burn vault tokens
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.vault_token_mint.to_account_info(),
            from: ctx.accounts.user_vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    burn(burn_ctx, vault_token_amount)?;

    // Update vault state
    vault.total_shares = vault
        .total_shares
        .checked_sub(vault_token_amount)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;
    
    vault.total_assets = vault
        .total_assets
        .checked_sub(expected_sol)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    // Create withdrawal ticket
    let ticket_id = clock.epoch;
    withdrawal_ticket.vault = vault.key();
    withdrawal_ticket.user = ctx.accounts.user.key();
    withdrawal_ticket.ticket_id = ticket_id;
    withdrawal_ticket.vault_tokens_burned = vault_token_amount;
    withdrawal_ticket.expected_sol_amount = expected_sol;
    withdrawal_ticket.request_epoch = clock.epoch;
    withdrawal_ticket.ready_to_claim = vault.buffered_sol >= expected_sol;
    withdrawal_ticket.claimed = false;
    withdrawal_ticket.bump = *ctx.bumps.get("withdrawal_ticket").unwrap();

    emit!(WithdrawalRequested {
        vault: vault.key(),
        user: ctx.accounts.user.key(),
        ticket_id,
        vault_tokens_burned: vault_token_amount,
        estimated_sol: expected_sol,
        timestamp: clock.unix_timestamp,
    });

    msg!("User {} requested withdrawal", ctx.accounts.user.key());
    msg!("Vault tokens burned: {}", vault_token_amount as f64 / 1e9);
    msg!("Expected SOL: {}", expected_sol as f64 / 1e9);
    msg!("Ready to claim: {}", withdrawal_ticket.ready_to_claim);

    Ok(())
}