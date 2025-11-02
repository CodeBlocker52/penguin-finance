use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct MintPsol<'info> {
    #[account(
        seeds = [FACTORY_SEED],
        bump = factory.bump,
    )]
    pub factory: Account<'info, Factory>,

    #[account(
        seeds = [VAULT_SEED, vault.factory.as_ref(), &vault.vault_id.to_le_bytes()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [PSOL_CONTROLLER_SEED],
        bump = psol_controller.bump,
    )]
    pub psol_controller: Account<'info, PsolController>,

    #[account(mut)]
    pub psol_mint: Account<'info, Mint>,

    /// Vault token mint for this vault
    pub vault_token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        space = UserPosition::LEN,
        seeds = [
            USER_POSITION_SEED,
            user.key().as_ref(),
            vault.key().as_ref()
        ],
        bump
    )]
    pub user_position: Account<'info, UserPosition>,

    /// User's vault token account (source of collateral)
    #[account(
        mut,
        associated_token::mint = vault_token_mint,
        associated_token::authority = user,
    )]
    pub user_vault_token_account: Account<'info, TokenAccount>,

    /// Position's vault token account (holds collateral)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = vault_token_mint,
        associated_token::authority = user_position,
    )]
    pub position_vault_token_account: Account<'info, TokenAccount>,

    /// User's pSOL token account (receives minted pSOL)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = psol_mint,
        associated_token::authority = user,
    )]
    pub user_psol_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<MintPsol>,
    collateral_amount: u64,
    psol_amount: u64,
) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);
    require!(collateral_amount > 0, ErrorCode::InvalidCollateralAmount);
    require!(psol_amount > 0, ErrorCode::InvalidPsolAmount);

    let vault = &ctx.accounts.vault;
    let psol_controller = &mut ctx.accounts.psol_controller;
    let user_position = &mut ctx.accounts.user_position;
    let clock = Clock::get()?;

    // Initialize position if new
    if user_position.owner == Pubkey::default() {
        user_position.owner = ctx.accounts.user.key();
        user_position.vault = vault.key();
        user_position.psol_controller = psol_controller.key();
        user_position.collateral_amount = 0;
        user_position.psol_debt = 0;
        user_position.last_update_epoch = clock.epoch;
        user_position.bump = ctx.bumps.user_position;
        
        psol_controller.active_positions = psol_controller
            .active_positions
            .checked_add(1)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
    }

    // Calculate collateral value
    let exchange_rate = vault.exchange_rate()?;
    let collateral_value = collateral_amount
        .checked_mul(exchange_rate)
        .and_then(|v| v.checked_div(1_000_000_000))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Calculate new collateralization ratio
    let new_collateral_total = user_position
        .collateral_amount
        .checked_add(collateral_amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    let new_debt_total = user_position
        .psol_debt
        .checked_add(psol_amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Check collateralization
    let new_collateral_value = new_collateral_total
        .checked_mul(exchange_rate)
        .and_then(|v| v.checked_div(1_000_000_000))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    let collateral_ratio = new_collateral_value
        .checked_mul(10000)
        .and_then(|v| v.checked_div(new_debt_total))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    require!(
        collateral_ratio >= psol_controller.min_collateral_ratio,
        ErrorCode::InsufficientCollateral
    );

    // Transfer vault tokens from user to position account
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_vault_token_account.to_account_info(),
            to: ctx.accounts.position_vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    transfer(transfer_ctx, collateral_amount)?;

    // Mint pSOL to user
    let controller_seeds = &[
        PSOL_CONTROLLER_SEED,
        &[psol_controller.bump],
    ];
    let signer_seeds = &[&controller_seeds[..]];

    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.psol_mint.to_account_info(),
            to: ctx.accounts.user_psol_account.to_account_info(),
            authority: psol_controller.to_account_info(),
        },
        signer_seeds,
    );
    mint_to(mint_ctx, psol_amount)?;

    // Update position
    user_position.collateral_amount = new_collateral_total;
    user_position.psol_debt = new_debt_total;
    user_position.last_update_epoch = clock.epoch;

    // Update controller
    psol_controller.total_psol_minted = psol_controller
        .total_psol_minted
        .checked_add(psol_amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    psol_controller.total_collateral_value = psol_controller
        .total_collateral_value
        .checked_add(collateral_value)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(PsolMinted {
        user: ctx.accounts.user.key(),
        vault: vault.key(),
        collateral_amount,
        psol_minted: psol_amount,
        collateral_ratio,
        timestamp: clock.unix_timestamp,
    });

    msg!("User {} minted {} pSOL", ctx.accounts.user.key(), psol_amount as f64 / 1e9);
    msg!("Collateral: {} vault tokens", collateral_amount as f64 / 1e9);
    msg!("Collateral ratio: {}%", collateral_ratio as f64 / 100.0);

    Ok(())
}