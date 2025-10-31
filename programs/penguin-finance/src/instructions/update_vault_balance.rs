use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct UpdateVaultBalance<'info> {
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

    /// Operator's vault token account (receives fee shares)
    #[account(
        init_if_needed,
        payer = oracle,
        associated_token::mint = vault_token_mint,
        associated_token::authority = vault.operator,
    )]
    pub operator_token_account: Account<'info, TokenAccount>,

    /// Treasury's vault token account (receives protocol fee shares)
    #[account(
        init_if_needed,
        payer = oracle,
        associated_token::mint = vault_token_mint,
        associated_token::authority = factory.treasury,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    /// Oracle/keeper that reports balances
    #[account(mut)]
    pub oracle: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<UpdateVaultBalance>, new_total_staked: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let factory = &ctx.accounts.factory;
    let clock = Clock::get()?;

    // Calculate rewards earned
    let old_total_staked = vault.total_staked;
    
    require!(
        new_total_staked >= old_total_staked,
        ErrorCode::InvalidVaultState
    );

    let rewards = new_total_staked
        .checked_sub(old_total_staked)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    if rewards == 0 {
        msg!("No rewards to distribute");
        return Ok(());
    }

    // Calculate fees
    let protocol_fee = rewards
        .checked_mul(factory.protocol_fee_bps as u64)
        .and_then(|v| v.checked_div(BASIS_POINTS_DIVISOR))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    let remaining_after_protocol = rewards
        .checked_sub(protocol_fee)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    let operator_fee = remaining_after_protocol
        .checked_mul(vault.fee_basis_points as u64)
        .and_then(|v| v.checked_div(BASIS_POINTS_DIVISOR))
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    let staker_rewards = remaining_after_protocol
        .checked_sub(operator_fee)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    // Calculate fee shares to mint
    let total_fee_amount = protocol_fee
        .checked_add(operator_fee)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    let fee_shares = if vault.total_shares > 0 {
        total_fee_amount
            .checked_mul(vault.total_shares)
            .and_then(|v| v.checked_div(vault.total_assets))
            .ok_or(ErrorCode::ArithmeticOverflow)?
    } else {
        0
    };

    // Split fee shares between protocol and operator
    let protocol_shares = if fee_shares > 0 {
        fee_shares
            .checked_mul(protocol_fee)
            .and_then(|v| v.checked_div(total_fee_amount))
            .ok_or(ErrorCode::ArithmeticOverflow)?
    } else {
        0
    };

    let operator_shares = fee_shares
        .checked_sub(protocol_shares)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    // Update vault state
    vault.total_staked = new_total_staked;
    vault.total_assets = vault
        .total_assets
        .checked_add(rewards)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    vault.total_shares = vault
        .total_shares
        .checked_add(fee_shares)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    vault.lifetime_rewards = vault
        .lifetime_rewards
        .checked_add(rewards)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    vault.last_reward_epoch = clock.epoch;

    // Mint fee shares
    let vault_seeds = &[
        VAULT_SEED,
        vault.factory.as_ref(),
        &vault.vault_id.to_le_bytes(),
        &[vault.bump],
    ];
    let signer_seeds = &[&vault_seeds[..]];

    if operator_shares > 0 {
        let mint_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.vault_token_mint.to_account_info(),
                to: ctx.accounts.operator_token_account.to_account_info(),
                authority: vault.to_account_info(),
            },
            signer_seeds,
        );
        mint_to(mint_ctx, operator_shares)?;
    }

    if protocol_shares > 0 {
        let mint_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.vault_token_mint.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: vault.to_account_info(),
            },
            signer_seeds,
        );
        mint_to(mint_ctx, protocol_shares)?;
    }

    let new_exchange_rate = vault.exchange_rate()?;

    emit!(RewardsDistributed {
        vault: vault.key(),
        epoch: clock.epoch,
        total_rewards: rewards,
        protocol_fee,
        operator_fee,
        staker_rewards,
        new_exchange_rate,
        timestamp: clock.unix_timestamp,
    });

    msg!("Rewards distributed for vault {}", vault.vault_id);
    msg!("Total rewards: {} SOL", rewards as f64 / 1e9);
    msg!("Protocol fee: {} SOL", protocol_fee as f64 / 1e9);
    msg!("Operator fee: {} SOL", operator_fee as f64 / 1e9);
    msg!("Staker rewards: {} SOL", staker_rewards as f64 / 1e9);
    msg!("New exchange rate: {}", new_exchange_rate as f64 / 1e9);

    Ok(())
}