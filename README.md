# Rewards Program

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built with Pinocchio](https://img.shields.io/badge/Built%20with-Pinocchio-purple)](https://github.com/solana-program/pinocchio)
[![Solana](https://img.shields.io/badge/Solana-Devnet-green)](https://solana.com)

## Program ID

```
REWArDioXgQJ2fZKkfu9LCLjQfRwYWVVfsvcsR5hoXi
```

## Deployments

| Network | Program ID |
| ------- | ---------- |

## Overview

A token rewards program for Solana that supports four distribution models: direct allocations with vesting, merkle-proof-based claims, continuous reward pools with proportional distribution, and authority-managed points.

## Key Features

- **Four distribution types** - Direct (on-chain recipient accounts), Merkle (off-chain tree, on-chain root), Continuous (proportional reward pools), and Points (authority-managed non-transferable tokens)
- **Configurable vesting schedules** - Immediate, Linear, Cliff, and CliffLinear (Direct and Merkle)
- **Continuous reward pools** - Users earn rewards proportional to their held balance over time
- **Points system** - Authority issues/uses/revokes non-transferable Token-2022 points with permanent delegate control
- **Revocation support** - Authority can revoke recipients across all distribution types (NonVested or Full mode)
- **Token-2022 support** - Works with both SPL Token and Token-2022 mints

## When to Use What

### Distribution Type

|                  | Direct                                          | Merkle                                                                   | Continuous                                                         | Points                                                       |
| ---------------- | ----------------------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------------------------ |
| **How it works** | Creates an on-chain account per recipient       | Stores a single merkle root on-chain; recipients provide proofs to claim | Users opt in; rewards distributed proportional to held balance     | Authority issues non-transferable Token-2022 tokens to users |
| **Upfront cost** | Authority pays rent for every recipient account | No per-recipient accounts until someone claims                           | Users pay rent for their own reward account on opt-in              | Payer pays rent for user ATA on first issue                  |
| **Scalability**  | Practical up to low thousands of recipients     | Scales to millions with constant on-chain storage                        | Scales to any number of opted-in users                             | Scales to any number of users                                |
| **Mutability**   | Recipients can be added after creation          | Recipient set is fixed at creation                                       | Users opt in/out freely; authority distributes rewards at any time | Authority issues/uses/revokes at any time                    |
| **Best for**     | Small, dynamic distributions                    | Large, fixed distributions                                               | Ongoing reward programs (staking, liquidity mining)                | Loyalty points, reputation, non-transferable rewards         |

### Vesting Schedule (Direct & Merkle only)

| Schedule        | Behavior                                                                                                                                          |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Immediate**   | All tokens are claimable right away                                                                                                               |
| **Linear**      | Tokens unlock proportionally between `start_ts` and `end_ts`                                                                                      |
| **Cliff**       | Nothing unlocks until `cliff_ts`, then everything unlocks at once                                                                                 |
| **CliffLinear** | Nothing unlocks until `cliff_ts`, then linear vesting from `start_ts` to `end_ts` (tokens accrued before the cliff become claimable at the cliff) |

### Revocation Modes

| Mode          | Behavior                                                                     |
| ------------- | ---------------------------------------------------------------------------- |
| **NonVested** | Vested/accrued tokens are transferred to the user; unvested tokens are freed |
| **Full**      | All tokens (vested and unvested) are returned to the authority               |

Revocation is opt-in per distribution via the `revocable` bitmask field. A `Revocation` marker PDA is created per user to permanently block future claims or opt-ins.

## Account Types

| Account            | PDA Seeds                                                     | Description                                   |
| ------------------ | ------------------------------------------------------------- | --------------------------------------------- |
| DirectDistribution | `["direct_distribution", mint, authority, seed]`              | Distribution config (authority, mint, totals) |
| DirectRecipient    | `["direct_recipient", distribution, recipient]`               | Recipient allocation and vesting schedule     |
| MerkleDistribution | `["merkle_distribution", mint, authority, seed]`              | Distribution config with merkle root          |
| MerkleClaim        | `["merkle_claim", distribution, claimant]`                    | Tracks claimed amount per claimant            |
| RewardPool         | `["reward_pool", reward_mint, tracked_mint, authority, seed]` | Continuous pool config and reward accumulator |
| UserRewardAccount  | `["user_reward", reward_pool, user]`                          | Tracks user participation and accrued rewards |
| PointsConfig       | `["points_config", authority, seed]`                          | Points system config (authority, flags)       |
| PointsMint         | `["mint", points_config]`                                     | Token-2022 mint with extensions (PDA)         |
| Revocation         | `["revocation", parent, user]`                                | Marker PDA blocking revoked users (all types) |

## Instructions

### Direct Distribution

| #   | Instruction              | Description                                         |
| --- | ------------------------ | --------------------------------------------------- |
| 0   | CreateDirectDistribution | Create distribution, fund vault                     |
| 1   | AddDirectRecipient       | Add recipient with vesting schedule                 |
| 2   | ClaimDirect              | Recipient claims vested tokens                      |
| 9   | RevokeDirectRecipient    | Authority revokes a recipient                       |
| 4   | CloseDirectRecipient     | Recipient reclaims rent after full vest             |
| 3   | CloseDirectDistribution  | Authority closes distribution, reclaims tokens/rent |

### Merkle Distribution

| #   | Instruction              | Description                                         |
| --- | ------------------------ | --------------------------------------------------- |
| 5   | CreateMerkleDistribution | Create distribution with merkle root, fund vault    |
| 6   | ClaimMerkle              | Claimant proves allocation and claims vested tokens |
| 10  | RevokeMerkleClaim        | Authority revokes a claimant with merkle proof      |
| 7   | CloseMerkleClaim         | Claimant reclaims rent after distribution closed    |
| 8   | CloseMerkleDistribution  | Authority closes distribution, reclaims tokens/rent |

### Continuous Reward Pool

| #   | Instruction                | Description                                              |
| --- | -------------------------- | -------------------------------------------------------- |
| 11  | CreateContinuousPool       | Create pool with tracked/reward mints and balance source |
| 12  | ContinuousOptIn            | User opts in; initial balance snapshot taken             |
| 14  | DistributeContinuousReward | Authority deposits rewards; accumulator updated          |
| 16  | SyncContinuousBalance      | Permissionless: sync user's on-chain token balance       |
| 17  | SetContinuousBalance       | Authority sets user balance (AuthoritySet mode only)     |
| 15  | ClaimContinuous            | User claims accrued rewards                              |
| 20  | SetContinuousMerkleRoot    | Authority sets/rotates merkle root for cumulative claims |
| 21  | ClaimContinuousMerkle      | User claims via merkle proof over cumulative amount      |
| 19  | RevokeContinuousUser       | Authority revokes user from pool                         |
| 13  | ContinuousOptOut           | User opts out and claims remaining rewards               |
| 18  | CloseContinuousPool        | Authority closes pool, reclaims remaining tokens         |

### Points

| #   | Instruction        | Description                                                  |
| --- | ------------------ | ------------------------------------------------------------ |
| 22  | InitPoints         | Create config PDA and Token-2022 mint with extensions        |
| 23  | IssuePoints        | Mint points to a user's ATA (created idempotently)           |
| 24  | UsePoints          | Burn points from user via permanent delegate (user cosigns)  |
| 25  | TransferPoints     | Transfer points between users via burn+mint (sender cosigns) |
| 28  | RevokePoints       | Authority force-burns user's entire balance                  |
| 26  | ClosePointsAccount | Close user's ATA after balance reaches zero (user cosigns)   |
| 27  | ClosePointsConfig  | Close config and mint, reclaim rent (supply must be 0)       |

The points mint is created with three Token-2022 extensions:

- **NonTransferable** — tokens cannot be transferred via standard Token-2022 transfers
- **PermanentDelegate** — the PointsConfig PDA can burn tokens from any holder
- **MintCloseAuthority** — the PointsConfig PDA can close the mint when supply is 0

Optional config flags: `transferable` (enables authority-mediated burn+mint transfers) and `revocable` (enables force-burn revocation).

Continuous pools also support cumulative-merkle claims for high-scale distribution accounting:

- Authority rotates snapshots with `SetContinuousMerkleRoot`
- Users claim deltas with `ClaimContinuousMerkle`
- Each rotation emits `MerkleRootSet` and each claim emits `Claimed`
- Full details: [Continuous Merkle Claim Mode](program/src/instructions/continuous/README.md#merkle-claim-mode)

## Workflow

### Direct Distribution

```mermaid
sequenceDiagram
    participant Authority
    participant Program
    participant Accounts

    Authority->>Program: CreateDirectDistribution
    Program->>Accounts: create Distribution PDA
    Program->>Accounts: create Vault ATA
    Program->>Accounts: transfer initial funding

    Authority->>Program: AddDirectRecipient
    Program->>Accounts: create Recipient PDA
    Program->>Accounts: update total_allocated
```

```mermaid
sequenceDiagram
    participant Recipient
    participant Program
    participant Accounts

    Note over Recipient,Accounts: time passes, tokens vest

    Recipient->>Program: ClaimDirect
    Program->>Accounts: calculate unlocked amount
    Program->>Recipient: transfer vested tokens
    Program->>Accounts: update claimed_amount
```

### Merkle Distribution

```mermaid
sequenceDiagram
    participant Authority
    participant Program
    participant Accounts

    Note over Authority: build merkle tree off-chain
    Authority->>Program: CreateMerkleDistribution (with root)
    Program->>Accounts: create Distribution PDA
    Program->>Accounts: create Vault ATA
    Program->>Accounts: transfer initial funding
```

```mermaid
sequenceDiagram
    participant Claimant
    participant Program
    participant Accounts

    Note over Claimant,Accounts: time passes, tokens vest

    Claimant->>Program: ClaimMerkle (with proof)
    Program->>Accounts: verify proof against root
    Program->>Accounts: create/update MerkleClaim PDA
    Program->>Claimant: transfer vested tokens
```

### Continuous Reward Pool

```mermaid
sequenceDiagram
    participant Authority
    participant Program
    participant User

    Authority->>Program: CreateContinuousPool
    Program->>Program: create RewardPool PDA + Vault ATA

    User->>Program: ContinuousOptIn
    Program->>Program: create UserRewardAccount PDA
    Program->>Program: snapshot initial balance

    Authority->>Program: DistributeContinuousReward
    Program->>Program: update reward_per_token accumulator

    User->>Program: ClaimContinuous
    Program->>Program: settle accrued rewards
    Program->>User: transfer reward tokens
```

### Points

```mermaid
sequenceDiagram
    participant Authority
    participant Program
    participant User

    Authority->>Program: InitPoints
    Program->>Program: create PointsConfig PDA
    Program->>Program: create Token-2022 mint (NonTransferable + PermanentDelegate + MintCloseAuthority)

    Authority->>Program: IssuePoints
    Program->>Program: create user ATA (idempotent)
    Program->>User: mint points to ATA

    Authority->>Program: UsePoints (+ user cosign)
    Program->>Program: burn points via permanent delegate

    Authority->>Program: ClosePointsAccount (+ user cosign)
    Program->>Program: verify zero balance
    Program->>User: close ATA, return rent
```

### Closing

```mermaid
sequenceDiagram
    participant Authority
    participant Program
    participant Accounts

    Authority->>Program: CloseDirectDistribution / CloseMerkleDistribution / CloseContinuousPool / ClosePointsConfig
    Program->>Accounts: return remaining tokens
    Program->>Accounts: close PDA
    Program->>Authority: reclaim rent
```

## Documentation

- [Direct Distribution](program/src/instructions/direct/README.md) - On-chain recipient accounts with vesting
- [Merkle Distribution](program/src/instructions/merkle/README.md) - Off-chain tree, on-chain root verification
- [Continuous Reward Pool](program/src/instructions/continuous/README.md) - Proportional balance-based rewards
- [Continuous Merkle Claim Mode](program/src/instructions/continuous/README.md#merkle-claim-mode) - Cumulative snapshot claims and root rotation
- [CU Benchmarks](docs/CU_BENCHMARKS.md) - Compute unit usage per instruction

## Local Development

### Prerequisites

- Rust
- Node.js (see `.nvmrc`)
- pnpm (see `package.json` `packageManager`)
- Solana CLI

All can be conveniently installed via the [Solana CLI Quick Install](https://solana.com/docs/intro/installation).

### Build & Test

```bash
# Install dependencies
just install

# Full build (IDL + clients + program)
just build

# Run integration tests
just integration-test

# Run integration tests with CU tracking
just integration-test --with-cu

# Format and lint
just fmt
```

## Tech Stack

- **[Pinocchio](https://github.com/anza-xyz/pinocchio)** - Lightweight `no_std` Solana framework
- **[Codama](https://github.com/codama-idl)** - IDL-driven client generation
- **[LiteSVM](https://github.com/LiteSVM/litesvm)** - Fast local testing

## Security Audit

`rewards` has been audited by [OtterSec](https://osec.io). View the [audit report](audits/2026-ottersec-solana-foundation-rewards-audit.pdf).

Audit status, audited-through commit, and the current unaudited delta are tracked in [audits/AUDIT_STATUS.md](audits/AUDIT_STATUS.md).

---

Built and maintained by the [Solana Foundation](https://solana.org/).

Licensed under MIT. See [LICENSE](LICENSE) for details.

## Support

- [**Solana StackExchange**](https://solana.stackexchange.com/) - tag `rewards-program`
- [**Open an Issue**](https://github.com/solana-program/rewards/issues/new)
