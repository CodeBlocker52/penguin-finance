# 🐧 Penguin Finance

**Decentralized Liquid Staking Protocol on Solana**

Penguin Finance is a permissionless liquid staking protocol that allows anyone to create staking vaults, stake SOL, and earn rewards while maintaining liquidity through liquid staking tokens (LSTs).

## 📋 Table of Contents

- [Introduction](#introduction)
- [Protocol Architecture](#protocol-architecture)
- [Key Features](#key-features)
- [Token Model](#token-model)
- [Project Structure](#project-structure)
- [User Flows](#user-flows)
- [Reward Mechanism](#reward-mechanism)
- [Security Features](#security-features)
- [Getting Started](#getting-started)

---

## 🎯 Introduction

### Problem Statement

Traditional Solana staking has several limitations:
- **Locked liquidity**: Staked SOL is illiquid with unbonding periods
- **High barriers**: Managing validators requires technical expertise
- **Centralization**: Limited validator choice in pooled solutions
- **Capital inefficiency**: Cannot use staked SOL in DeFi

### Solution

Penguin Finance introduces a **vault-based liquid staking marketplace**:
- **Permissionless Vaults**: Anyone can create a staking vault
- **Liquid Staking Tokens**: Receive tradeable tokens representing staked SOL
- **Overcollateralized pSOL**: Unified liquid token backed by vault tokens
- **Market-Driven**: Operators compete on fees and performance
- **Isolated Risk**: Each vault operates independently

---

## 🏗️ Protocol Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────┐
│              Penguin Finance Protocol                    │
│         (Factory & Core Smart Contracts)                 │
└───────────────────┬─────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
┌───────▼────────┐      ┌──────▼─────────┐
│   Vault A      │      │   Vault B      │
│ Solo Operator  │      │ Institution    │
│ Fee: 5%        │      │ Fee: 8%        │
│ Cap: 10K SOL   │      │ Cap: 100K SOL  │
└───────┬────────┘      └──────┬─────────┘
        │                      │
        │ Issues               │ Issues
        │ pVault-A             │ pVault-B
        │ tokens               │ tokens
        │                      │
┌───────▼────────┐      ┌──────▼─────────┐
│ User Deposits  │      │ User Deposits  │
└───────┬────────┘      └──────┬─────────┘
        │                      │
        │ Delegates            │ Delegates
        │ to Validators        │ to Validators
        │                      │
        └──────────┬───────────┘
                   │
        ┌──────────▼────────────┐
        │   Solana Validators   │
        │   Earn Staking Rewards │
        └──────────┬────────────┘
                   │
        ┌──────────▼────────────┐
        │  pSOL (Global Token)  │
        │  Overcollateralized   │
        │  DeFi Composable      │
        └───────────────────────┘
```

### Core Components

#### 1. **Factory Contract**
- Deploys new staking vaults
- Maintains vault registry
- Enforces protocol parameters
- Collects protocol fees

#### 2. **Vault Contracts**
- Independent staking pools
- Operator-managed
- Custom fee structures
- Isolated risk profiles
- Issue vault-specific LST tokens

#### 3. **pSOL Controller**
- Manages pSOL minting/burning
- Enforces overcollateralization
- Handles liquidations
- Aggregates vault rewards

#### 4. **Reward Oracle**
- Reports validator performance
- Calculates epoch rewards
- Updates vault exchange rates
- Consensus mechanism

---

## ✨ Key Features

### For Stakers

- ✅ **Any Amount**: Stake from 0.1 SOL (no minimum)
- ✅ **Instant Liquidity**: Receive tradeable LST tokens immediately
- ✅ **Vault Choice**: Select operators based on fees, performance, reputation
- ✅ **Diversification**: Spread stake across multiple vaults
- ✅ **DeFi Integration**: Use pSOL as collateral, LP, or in lending protocols
- ✅ **Compounding**: Automatic reward reinvestment

### For Operators

- ✅ **Permissionless**: Anyone can create a vault
- ✅ **Earn Fees**: Set custom fee structure (typically 5-10%)
- ✅ **Build Reputation**: Performance tracked on-chain
- ✅ **Flexible Capacity**: Set max vault size
- ✅ **Validator Control**: Choose which validators to delegate to
- ✅ **White-Label**: Custom branding and parameters

### Protocol Features

- ✅ **Risk Isolation**: Vault failures don't affect others
- ✅ **Overcollateralization**: pSOL backed by 110%+ of vault tokens
- ✅ **Decentralized**: No central operator whitelist
- ✅ **Composable**: Standard SPL token interface
- ✅ **Upgradeable**: Managed through governance

---

## 🪙 Token Model

### Two-Tier Token System

#### Tier 1: Vault Tokens (pVault-{ID})

**Characteristics:**
- Issued when users deposit SOL into specific vault
- Represents proportional ownership of vault's staked SOL
- Non-rebasing: value increases as rewards accrue
- Vault-specific: pVault-1 ≠ pVault-2
- Redeemable for SOL from originating vault

**Example:**
```
Vault #42 has:
- Total Assets: 1,000 SOL (staked + rewards)
- Total Supply: 950 pVault-42 tokens
- Exchange Rate: 1.0526 SOL per pVault-42

User deposits 10 SOL
Receives: 9.5 pVault-42 tokens
```

#### Tier 2: pSOL (Penguin SOL)

**Characteristics:**
- Unified liquid staking token
- Minted by locking vault tokens as collateral
- Overcollateralized (minimum 110% collateral ratio)
- Fungible across all vaults
- Optimized for DeFi integration
- Value accrues from underlying vault rewards

**Collateralization Example:**
```
User has: 10 pVault-42 (worth 10.526 SOL)
Mints: 9.5 pSOL
Collateral Ratio: 110.8%

Collateral continues earning staking rewards
User can use pSOL in DeFi applications
```

### Token Flow

```
SOL → Vault Deposit → pVault-X tokens
                           ↓
                    Lock as Collateral
                           ↓
                      Mint pSOL
                           ↓
                    Use in DeFi
                           ↓
                      Burn pSOL
                           ↓
                   Unlock pVault-X
                           ↓
                     Redeem for SOL
```

---

## 📁 Project Structure

```
penguin-finance/
├── programs/
│   └── penguin-finance/
│       ├── src/
│       │   ├── lib.rs                 # Program entry point
│       │   ├── state/
│       │   │   ├── mod.rs
│       │   │   ├── factory.rs         # Protocol factory state
│       │   │   ├── vault.rs           # Vault state
│       │   │   ├── vault_token.rs     # Vault LST metadata
│       │   │   ├── psol_controller.rs # pSOL controller state
│       │   │   └── user_position.rs   # User collateral positions
│       │   ├── instructions/
│       │   │   ├── mod.rs
│       │   │   ├── initialize_factory.rs
│       │   │   ├── create_vault.rs
│       │   │   ├── deposit_to_vault.rs
│       │   │   ├── stake_from_vault.rs
│       │   │   ├── update_vault_balance.rs
│       │   │   ├── mint_psol.rs
│       │   │   ├── burn_psol.rs
│       │   │   ├── request_withdrawal.rs
│       │   │   └── claim_withdrawal.rs
│       │   ├── errors.rs              # Custom error codes
│       │   ├── events.rs              # Event emissions
│       │   └── constants.rs           # Protocol constants
│       └── Cargo.toml
├── tests/
│   ├── penguin-finance.ts             # Main test suite
│   ├── vault-operations.ts            # Vault tests
│   ├── psol-minting.ts                # pSOL tests
│   └── rewards.ts                     # Reward distribution tests
├── migrations/
│   └── deploy.ts                      # Deployment script
├── app/                               # Frontend (optional)
├── Anchor.toml                        # Anchor configuration
├── package.json
└── README.md
```

### Key Directories

- **`state/`**: Account structures and data models
- **`instructions/`**: Business logic for each operation
- **`tests/`**: Comprehensive test coverage
- **`errors.rs`**: Custom error definitions
- **`events.rs`**: Event logs for indexing

---

## 👥 User Flows

### Flow 1: Operator Creates Vault

```
1. Operator calls create_vault()
   └─ Parameters:
      ├─ fee_basis_points: u16 (e.g., 500 = 5%)
      ├─ max_capacity: u64 (in lamports)
      └─ name: String

2. Protocol deploys new vault PDA
   └─ Seeds: ["vault", factory, vault_id]

3. Vault token mint created
   └─ Authority: vault PDA

4. Operator registered as vault authority

5. Vault listed in factory registry
```

### Flow 2: User Stakes SOL

```
1. User selects vault (based on fees, performance, trust)

2. User calls deposit_to_vault()
   └─ Amount: 10 SOL

3. SOL transferred to vault account

4. Vault calculates shares:
   shares = amount × total_shares / total_assets
   └─ If first deposit: shares = amount

5. Vault mints pVault tokens to user
   └─ User receives: 9.5 pVault-42 tokens

6. Vault's total_assets += 10 SOL
   └─ total_shares += 9.5

7. Event emitted: DepositMade
```

### Flow 3: Vault Stakes to Validators

```
1. Operator calls stake_from_vault()
   └─ validator_vote_account: Pubkey
   └─ amount: u64

2. Vault creates stake account
   └─ Authority: vault PDA
   └─ Withdraw authority: vault PDA

3. Delegates stake to validator

4. Stake activates next epoch

5. Validators begin earning rewards

6. Event emitted: StakeDelegated
```

### Flow 4: Mint pSOL (Overcollateralized Token)

```
1. User has 10 pVault-42 tokens (worth 10.5 SOL)

2. User calls mint_psol()
   └─ collateral_amount: 10 pVault-42
   └─ psol_amount: 9.5 pSOL

3. Check collateralization:
   collateral_value = 10 × exchange_rate = 10.5 SOL
   psol_value = 9.5 SOL
   ratio = 10.5 / 9.5 = 110.5% ✓ (min: 110%)

4. Transfer pVault tokens to user position account

5. Mint 9.5 pSOL to user

6. Record position:
   └─ collateral: 10 pVault-42
   └─ debt: 9.5 pSOL

7. User can now use pSOL in DeFi

8. Event emitted: PsolMinted
```

### Flow 5: Reward Distribution

```
1. End of epoch, validators earn rewards

2. Oracle calls update_vault_balance()
   └─ Reports new stake account balances

3. Vault calculates rewards:
   rewards = new_balance - old_balance

4. Apply operator fee:
   operator_fee = rewards × fee_basis_points / 10000
   └─ Mint fee shares to operator

5. Update exchange rate:
   total_assets += rewards
   └─ User vault tokens now worth more

6. pSOL value increases proportionally
   └─ Backed by more valuable collateral

7. Event emitted: RewardsDistributed
```

### Flow 6: Withdrawal Process

```
1. User calls burn_psol()
   └─ Burns pSOL debt
   └─ Unlocks pVault collateral

2. User calls request_withdrawal()
   └─ Burns pVault tokens
   └─ Creates withdrawal ticket NFT

3. Vault must deactivate stake:
   └─ If buffered SOL available: instant
   └─ Else: unstake from validator (waits 1-2 epochs)

4. When SOL available, user calls claim_withdrawal()
   └─ Burns ticket NFT
   └─ Receives SOL

5. Event emitted: WithdrawalCompleted
```

---

## 💰 Reward Mechanism

### How Rewards Are Generated

#### Validator-Level Rewards

Solana validators earn from:
1. **Inflation Rewards**: ~5-7% APY from protocol inflation
2. **Transaction Fees**: Priority fees from users
3. **MEV**: Potential additional yield from transaction ordering

#### Vault-Level Collection

```
Epoch N ends:
├─ Validator earned: 100 SOL in rewards
├─ Vault's stake: 10,000 SOL (10% of validator)
└─ Vault's share: 10 SOL
```

### Distribution Waterfall

```
Gross Rewards: 10 SOL
        ↓
Protocol Fee (1%): 0.1 SOL → Treasury
        ↓
Remaining: 9.9 SOL
        ↓
Operator Fee (5%): 0.495 SOL → Vault Operator
        ↓
Net to Stakers: 9.405 SOL
        ↓
Exchange Rate Increase
        ↓
All pVault token holders benefit proportionally
```

### Exchange Rate Calculation

```rust
// Before rewards
total_assets = 10,000 SOL
total_shares = 9,500 pVault tokens
exchange_rate = 10,000 / 9,500 = 1.0526 SOL per token

// After rewards (+9.405 SOL net)
total_assets = 10,009.405 SOL
total_shares = 9,500 pVault tokens (unchanged)
exchange_rate = 10,009.405 / 9,500 = 1.0536 SOL per token

// Growth: +0.0010 SOL per token
// APY: ~9.4% (annualized from epoch rewards)
```

### User Reward Accrual

**Example:**
```
User holds: 100 pVault-42 tokens

Beginning of epoch:
  Value: 100 × 1.0526 = 105.26 SOL

End of epoch:
  Value: 100 × 1.0536 = 105.36 SOL

Earned: 0.10 SOL (no action needed)
```

### pSOL Value Appreciation

Since pSOL is backed by vault tokens:
```
pSOL exchange rate = Σ(vault_collateral_value) / total_psol_supply

As vault tokens appreciate:
└─ pSOL value increases proportionally
   └─ Holders earn staking rewards while using pSOL in DeFi
```

---

## 🔒 Security Features

### Vault Isolation

- Each vault is separate PDA
- Slashing in one vault doesn't affect others
- Users can avoid underperforming vaults
- Operators have skin in the game

### Overcollateralization

```
Health Factor = collateral_value / debt_value

Safe: ≥ 1.10 (110%)
Warning: < 1.10
Liquidatable: < 1.05
```

**Liquidation Process:**
1. Anyone can trigger liquidation of unhealthy positions
2. Liquidator repays debt (pSOL)
3. Receives collateral (vault tokens) at discount
4. Protects pSOL peg and solvency

### Authority Controls

```
Vault Authority Hierarchy:
├─ Vault PDA (program-derived)
│  ├─ Controls stake accounts
│  ├─ Manages withdrawals
│  └─ Cannot be controlled by operator
├─ Operator Account
│  ├─ Can delegate stake
│  ├─ Can update vault parameters
│  └─ Cannot withdraw user funds
└─ Factory Authority (Governance)
   ├─ Can pause protocol
   ├─ Can update fees
   └─ Cannot access user funds
```

### Withdrawal Credentials

- Stake accounts created with vault PDA as withdraw authority
- Operators cannot steal staked SOL
- Only vault PDA can initiate withdrawals
- Enforced at stake program level

### Time Locks

- Governance changes: 3-day timelock
- Fee increases: 7-day notice
- Emergency pause: Immediate (multisig)

---

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Install Anchor 0.31.1
cargo install --git https://github.com/coral-xyz/anchor --tag v0.31.1 anchor-cli
```

### Installation

```bash
# Clone repository
git clone https://github.com/your-org/penguin-finance
cd penguin-finance

# Install dependencies
npm install

# Build program
anchor build

# Run tests
anchor test
```

### Deployment

```bash
# Configure Solana to devnet
solana config set --url devnet

# Airdrop SOL for testing
solana airdrop 2

# Deploy program
anchor deploy

# Initialize factory
anchor run initialize
```

### Configuration

Edit `Anchor.toml`:
```toml
[programs.devnet]
penguin_finance = "YourProgramIDHere..."

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"
```

---

## 📊 Protocol Parameters

| Parameter | Value | Adjustable |
|-----------|-------|------------|
| Minimum Collateral Ratio | 110% | ✅ Governance |
| Liquidation Threshold | 105% | ✅ Governance |
| Protocol Fee | 1% | ✅ Governance |
| Maximum Vault Fee | 15% | ✅ Governance |
| Minimum Stake | 0.1 SOL | ✅ Governance |
| Withdrawal Delay | 1-2 epochs | ❌ Solana Protocol |

---

## 🎯 Roadmap

### Phase 1: Core Protocol (Hackathon)
- ✅ Factory and vault contracts
- ✅ Vault token minting
- ✅ Basic staking delegation
- ✅ pSOL minting/burning
- ✅ Reward distribution
- ✅ Test suite

### Phase 2: Enhanced Features
- 🔄 Oracle integration for automated rewards
- 🔄 Liquidation engine
- 🔄 Frontend dashboard
- 🔄 Vault marketplace UI

### Phase 3: DeFi Integration
- 🔄 Lending protocol integration
- 🔄 DEX liquidity pools
- 🔄 Cross-chain bridges
- 🔄 Governance token launch

### Phase 4: Decentralization
- 🔄 DAO governance
- 🔄 Protocol-owned liquidity
- 🔄 Community-driven development

---




**Built with 🐧 by the Penguin Finance Team**