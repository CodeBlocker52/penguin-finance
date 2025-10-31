
use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("GQtLSrEgqfhcETMzQcP4dX2Sgv3WEnxDnCcZCZg6a9m4");

#[program]
pub mod penguin_finance {
    use super::*;

    /// Initialize the protocol factory
    pub fn initialize_factory(ctx: Context<InitializeFactory>) -> Result<()> {
        instructions::initialize_factory::handler(ctx)
    }

    /// Create a new staking vault
    pub fn create_vault(
        ctx: Context<CreateVault>,
        fee_basis_points: u16,
        max_capacity: u64,
        vault_name: String,
    ) -> Result<()> {
        instructions::create_vault::handler(ctx, fee_basis_points, max_capacity, vault_name)
    }

    /// Deposit SOL into a vault and receive vault tokens
    pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
        instructions::deposit_to_vault::handler(ctx, amount)
    }

    /// Stake SOL from vault to a validator
    pub fn stake_from_vault(
        ctx: Context<StakeFromVault>,
        amount: u64,
    ) -> Result<()> {
        instructions::stake_from_vault::handler(ctx, amount)
    }

    /// Update vault balance and distribute rewards
    pub fn update_vault_balance(
        ctx: Context<UpdateVaultBalance>,
        new_total_staked: u64,
    ) -> Result<()> {
        instructions::update_vault_balance::handler(ctx, new_total_staked)
    }

    /// Mint pSOL using vault tokens as collateral
    pub fn mint_psol(
        ctx: Context<MintPsol>,
        collateral_amount: u64,
        psol_amount: u64,
    ) -> Result<()> {
        instructions::mint_psol::handler(ctx, collateral_amount, psol_amount)
    }

    /// Burn pSOL to unlock vault token collateral
    pub fn burn_psol(ctx: Context<BurnPsol>, psol_amount: u64) -> Result<()> {
        instructions::burn_psol::handler(ctx, psol_amount)
    }

    /// Request withdrawal from vault
    pub fn request_withdrawal(
        ctx: Context<RequestWithdrawal>,
        vault_token_amount: u64,
    ) -> Result<()> {
        instructions::request_withdrawal::handler(ctx, vault_token_amount)
    }

    /// Claim completed withdrawal
    pub fn claim_withdrawal(ctx: Context<ClaimWithdrawal>) -> Result<()> {
        instructions::claim_withdrawal::handler(ctx)
    }

    /// Liquidate unhealthy pSOL position
    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        instructions::liquidate_position::handler(ctx)
    }
}