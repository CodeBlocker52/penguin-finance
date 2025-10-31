use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
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

    /// Vault token mint
    pub vault_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [PSOL_CONTROLLER_SEED],
        bump = psol_controller.bump,
    )]
    pub psol_controller: Account<'info, PsolController>,

    #[account(mut)]
    pub psol_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            USER_POSITION_SEED,
            user_position.owner.as_ref(),
            vault.key().as_ref()
        ],
        bump = user_position.bump,
    )]
    pub user_position: Account<'info, UserPosition>,

    /// Position's vault token account (holds collateral)
    #[account(
        mut,
        associated_token::mint = vault_token_mint,
        associated_token::authority = user_position,
    )]
    pub position_vault_token_account: Account<'info, TokenAccount>,

    /// Liquidator's pSOL account (pays debt)
    #[account(
        mut,
        associated_token::mint = psol_mint,
        associated_token::authority = liquidator,
    )]
    pub liquidator_psol_account: Account<'info, TokenAccount>,

    /// Liquidator's vault token account (receives collateral)
    #[account(
        mut,
        associated_token::mint = vault_token_mint,
        associated_token::authority = liquidator,
    )]
    pub liquidator_vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub liquidator: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<LiquidatePosition>) -> Result<()> {
    require!(!ctx.accounts.factory.paused, ErrorCode::VaultPaused);

    let vault = &ctx.accounts.vault;
    let psol_controller = &mut ctx.accounts.psol_controller;
    let user_position = &mut ctx.accounts.user_position;
    let clock = Clock::get()?;

    // Check if position is liquidatable
    let exchange_rate = vault.exchange_rate()?;
    let is_liquidatable = user_position.is_liquidatable(
        exchange_rate,
        psol_controller.liquidation_threshold,
    )?;

    require!(is_liquidatable, ErrorCode::PositionHealthy);

    let debt = user_position.psol_debt;
    let collateral = user_position.collateral_amount;

    // Calculate liquidation bonus
    let bonus_amount = collateral
        .checked_mul(psol_controller.liquidation_bonus)
        .and_then(|v| v.checked_div(BASIS_POINTS_DIVISOR))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    let total_collateral_to_liquidator = collateral;

    // Burn pSOL from liquidator
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.psol_mint.to_account_info(),
            from: ctx.accounts.liquidator_psol_account.to_account_info(),
            authority: ctx.accounts.liquidator.to_account_info(),
        },
    );
    burn(burn_ctx, debt)?;

    // Transfer collateral to liquidator
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
            to: ctx.accounts.liquidator_vault_token_account.to_account_info(),
            authority: user_position.to_account_info(),
        },
        signer_seeds,
    );
    transfer(transfer_ctx, total_collateral_to_liquidator)?;

    // Calculate collateral value
    let collateral_value = collateral
        .checked_mul(exchange_rate)
        .and_then(|v| v.checked_div(1_000_000_000))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Update position (zeroed out)
    user_position.collateral_amount = 0;
    user_position.psol_debt = 0;
    user_position.last_update_epoch = clock.epoch;

    // Update controller
    psol_controller.total_psol_minted = psol_controller
        .total_psol_minted
        .checked_sub(debt)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;
    
    psol_controller.total_collateral_value = psol_controller
        .total_collateral_value
        .checked_sub(collateral_value)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;
    
    psol_controller.active_positions = psol_controller
        .active_positions
        .checked_sub(1)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    emit!(PositionLiquidated {
        liquidator: ctx.accounts.liquidator.key(),
        position_owner: user_position.owner,
        vault: vault.key(),
        collateral_seized: total_collateral_to_liquidator,
        debt_repaid: debt,
        liquidation_bonus: bonus_amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Position liquidated");
    msg!("Debt repaid: {} pSOL", debt as f64 / 1e9);
    msg!("Collateral seized: {} vault tokens", total_collateral_to_liquidator as f64 / 1e9);

    Ok(())
}