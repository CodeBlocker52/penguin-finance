use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct ClaimWithdrawal<'info> {
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

    #[account(
        mut,
        seeds = [
            WITHDRAWAL_TICKET_SEED,
            withdrawal_ticket.vault.as_ref(),
            user.key().as_ref(),
            &withdrawal_ticket.ticket_id.to_le_bytes()
        ],
        bump = withdrawal_ticket.bump,
        constraint = withdrawal_ticket.user == user.key() @ ErrorCode::Unauthorized,
        constraint = !withdrawal_ticket.claimed @ ErrorCode::InvalidWithdrawalTicket,
    )]
    pub withdrawal_ticket: Account<'info, WithdrawalTicket>,

    /// CHECK: User receiving SOL
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimWithdrawal>) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);

    let vault = &mut ctx.accounts.vault;
    let withdrawal_ticket = &mut ctx.accounts.withdrawal_ticket;
    let clock = Clock::get()?;

    // Check if withdrawal is ready
    let is_ready = vault.buffered_sol >= withdrawal_ticket.expected_sol_amount;
    
    require!(is_ready, ErrorCode::WithdrawalNotReady);

    let sol_amount = withdrawal_ticket.expected_sol_amount;

    // Transfer SOL from vault to user
    let vault_seeds = &[
        VAULT_SEED,
        vault.factory.as_ref(),
        &vault.vault_id.to_le_bytes(),
        &[vault.bump],
    ];
    let signer_seeds = &[&vault_seeds[..]];

    // Use invoke_signed to transfer SOL
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::transfer(
            &vault.key(),
            &ctx.accounts.user.key(),
            sol_amount,
        ),
        &[
            vault.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    // Update vault state
    vault.buffered_sol = vault
        .buffered_sol
        .checked_sub(sol_amount)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    // Mark ticket as claimed
    withdrawal_ticket.claimed = true;
    withdrawal_ticket.ready_to_claim = true;

    emit!(WithdrawalCompleted {
        vault: vault.key(),
        user: ctx.accounts.user.key(),
        ticket_id: withdrawal_ticket.ticket_id,
        sol_amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrawal claimed by {}", ctx.accounts.user.key());
    msg!("SOL amount: {}", sol_amount as f64 / 1e9);

    Ok(())
}