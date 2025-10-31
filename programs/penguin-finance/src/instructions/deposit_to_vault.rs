use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct DepositToVault<'info> {
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
        has_one = vault_token_mint,
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub vault_token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = vault_token_mint,
        associated_token::authority = user,
    )]
    pub user_vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);
    require!(ctx.accounts.vault.accepting_deposits, ErrorCode::VaultPaused);
    require!(amount >= MIN_STAKE_AMOUNT, ErrorCode::DepositTooSmall);
    require!(
        ctx.accounts.vault.has_capacity(amount),
        ErrorCode::VaultCapacityReached
    );

    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Calculate shares to mint
    let shares_to_mint = vault.calculate_shares(amount)?;
    
    // Get exchange rate before deposit for event
    let exchange_rate = vault.exchange_rate()?;

    // Transfer SOL from user to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: vault.to_account_info(),
        },
    );
    transfer(transfer_ctx, amount)?;

    // Update vault state
    vault.buffered_sol = vault
        .buffered_sol
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    vault.total_assets = vault
        .total_assets
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    vault.total_shares = vault
        .total_shares
        .checked_add(shares_to_mint)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Mint vault tokens to user
    let vault_seeds = &[
        VAULT_SEED,
        vault.factory.as_ref(),
        &vault.vault_id.to_le_bytes(),
        &[vault.bump],
    ];
    let signer_seeds = &[&vault_seeds[..]];

    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.vault_token_mint.to_account_info(),
            to: ctx.accounts.user_vault_token_account.to_account_info(),
            authority: vault.to_account_info(),
        },
        signer_seeds,
    );
    mint_to(mint_ctx, shares_to_mint)?;

    emit!(DepositMade {
        vault: vault.key(),
        user: ctx.accounts.user.key(),
        sol_amount: amount,
        vault_tokens_minted: shares_to_mint,
        exchange_rate,
        timestamp: clock.unix_timestamp,
    });

    msg!("User {} deposited {} SOL", ctx.accounts.user.key(), amount as f64 / 1e9);
    msg!("Minted {} vault tokens", shares_to_mint as f64 / 1e9);
    msg!("Exchange rate: {}", exchange_rate as f64 / 1e9);

    Ok(())
}