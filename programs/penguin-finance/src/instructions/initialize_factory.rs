use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::constants::*;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeFactory<'info> {
    #[account(
        init,
        payer = authority,
        space = Factory::LEN,
        seeds = [FACTORY_SEED],
        bump
    )]
    pub factory: Account<'info, Factory>,

    #[account(
        init,
        payer = authority,
        space = PsolController::LEN,
        seeds = [PSOL_CONTROLLER_SEED],
        bump
    )]
    pub psol_controller: Account<'info, PsolController>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = psol_controller,
        seeds = [b"psol_mint"],
        bump
    )]
    pub psol_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Treasury can be any account
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeFactory>) -> Result<()> {
    let clock = Clock::get()?;
    
    let factory = &mut ctx.accounts.factory;
    let factory_key = factory.key();
    let psol_controller_key = ctx.accounts.psol_controller.key();

    // Initialize factory
    factory.authority = ctx.accounts.authority.key();
    factory.treasury = ctx.accounts.treasury.key();
    factory.vault_count = 0;
    factory.protocol_fee_bps = PROTOCOL_FEE_BPS;
    factory.paused = false;
    factory.psol_mint = ctx.accounts.psol_mint.key();
    factory.psol_controller = psol_controller_key;
    factory.bump = ctx.bumps.factory;

    // Initialize pSOL controller
    let psol_controller = &mut ctx.accounts.psol_controller;
    psol_controller.factory = factory_key;
    psol_controller.psol_mint = ctx.accounts.psol_mint.key();
    psol_controller.total_psol_minted = 0;
    psol_controller.total_collateral_value = 0;
    psol_controller.min_collateral_ratio = MIN_COLLATERAL_RATIO;
    psol_controller.liquidation_threshold = LIQUIDATION_THRESHOLD;
    psol_controller.liquidation_bonus = LIQUIDATION_BONUS;
    psol_controller.active_positions = 0;
    psol_controller.bump = ctx.bumps.psol_controller;

    emit!(FactoryInitialized {
        factory: factory_key,
        authority: ctx.accounts.authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Protocol factory initialized");
    msg!("pSOL mint: {}", ctx.accounts.psol_mint.key());
    
    Ok(())
}