use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct BurnPsol<'info> {
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
        has_one = factory,
        has_one = psol_mint,
    )]
    pub psol_controller: Account<'info, PsolController>,

    #[account(mut)]
    pub psol_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            USER_POSITION_SEED,
            user.key().as_ref(),
            vault.key().as_ref()
        ],
        bump = user_position.bump,
        has_one = vault,
        has_one = psol_controller,
        constraint = user_position.owner == user.key() @ ErrorCode::Unauthorized,
    )]
    pub user_position: Account<'info, UserPosition>,

    /// User's vault token account (receives collateral back)
    #[account(
        mut,
        associated_token::mint = vault.vault_token_mint,
        associated_token::authority = user,
    )]
    pub user_vault_token_account: Account<'info, TokenAccount>,

    /// Position's vault token account (holds collateral)
    #[account(
        mut,
        associated_token::mint = vault.vault_token_mint,
        associated_token::authority = user_position,
    )]
    pub position_vault_token_account: Account<'info, TokenAccount>,

    /// User's pSOL token account (source of pSOL to burn)
    #[account(
        mut,
        associated_token::mint = psol_mint,
        associated_token::authority = user,
    )]
    pub user_psol_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BurnPsol>, psol_amount: u64) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);
    require!(psol_amount > 0, ErrorCode::InvalidPsolAmount);

    let vault = &ctx.accounts.vault;
    let psol_controller = &mut ctx.accounts.psol_controller;
    let user_position = &mut ctx.accounts.user_position;
    let clock = Clock::get()?;

    require!(
        user_position.psol_debt >= psol_amount,
        ErrorCode::InvalidPsolAmount
    );

    // Calculate collateral to release proportionally
    let collateral_to_release = if user_position.psol_debt == psol_amount {
        // Full repayment, release all collateral
        user_position.collateral_amount
    } else {
        // Partial repayment, release proportional collateral
        user_position
            .collateral_amount
            .checked_mul(psol_amount)
            .and_then(|v| v.checked_div(user_position.psol_debt))
            .ok_or(ErrorCode::ArithmeticOverflow)?
    };

    // Burn pSOL from user
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.psol_mint.to_account_info(),
            from: ctx.accounts.user_psol_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    burn(burn_ctx, psol_amount)?;

    // Transfer collateral back to user
    let position_seeds = &[
        USER_POSITION_SEED,
        user_position.owner.as_ref(),
        user_position.vault.as_ref(),
        &[user_position.bump],
    ];
    let signer_seeds = &[&position_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.position_vault_token_account.to_account_info(),
            to: ctx.accounts.user_vault_token_account.to_account_info(),
            authority: user_position.to_account_info(),
        },
        signer_seeds,
    );
    transfer(transfer_ctx, collateral_to_release)?;

    // Calculate collateral value released
    let exchange_rate = vault.exchange_rate()?;
    let collateral_value = collateral_to_release
        .checked_mul(exchange_rate)
        .and_then(|v| v.checked_div(1_000_000_000))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Update position
    user_position.collateral_amount = user_position
        .collateral_amount
        .checked_sub(collateral_to_release)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;
    
    user_position.psol_debt = user_position
        .psol_debt
        .checked_sub(psol_amount)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;
    
    user_position.last_update_epoch = clock.epoch;

    // Update controller
    psol_controller.total_psol_minted = psol_controller
        .total_psol_minted
        .checked_sub(psol_amount)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;
    
    psol_controller.total_collateral_value = psol_controller
        .total_collateral_value
        .checked_sub(collateral_value)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    // If position is fully closed, decrement active positions
    if user_position.psol_debt == 0 && user_position.collateral_amount == 0 {
        psol_controller.active_positions = psol_controller
            .active_positions
            .checked_sub(1)
            .ok_or(ErrorCode::ArithmeticUnderflow)?;
    }

    emit!(PsolBurned {
        user: ctx.accounts.user.key(),
        vault: vault.key(),
        psol_burned: psol_amount,
        collateral_released: collateral_to_release,
        timestamp: clock.unix_timestamp,
    });

    msg!("User {} burned {} pSOL", ctx.accounts.user.key(), psol_amount as f64 / 1e9);
    msg!("Collateral released: {} vault tokens", collateral_to_release as f64 / 1e9);

    Ok(())
}