# Continuous Reward Pool

Proportional reward distribution where users earn based on their held token balance over time.

## How It Works

The authority creates a `RewardPool` that tracks a token mint (e.g. USD1) and distributes rewards in a separate reward mint. Users opt in to start earning. When the authority calls `DistributeContinuousReward`, rewards are split proportionally across all opted-in users based on their tracked balance, using a gas-efficient reward-per-token accumulator (similar to Synthetix StakingRewards).

### Balance Source Modes

- **OnChain**: User balances are read directly from their SPL token account for the tracked mint. Anyone can call `SyncContinuousBalance` to update a user's balance. Claims and opt-outs auto-sync.
- **AuthoritySet**: The authority acts as an oracle, setting user balances via `SetContinuousBalance`. Useful for off-chain or cross-chain data.

## Account Structure

| Account             | PDA Seeds                                                     | Description                                   |
| ------------------- | ------------------------------------------------------------- | --------------------------------------------- |
| `RewardPool`        | `["reward_pool", reward_mint, tracked_mint, authority, seed]` | Pool config and reward accumulator            |
| `UserRewardAccount` | `["user_reward", reward_pool, user]`                          | Tracks user participation and accrued rewards |
| `MerkleClaim`       | `["merkle_claim", reward_pool, user]`                         | Tracks cumulative merkle-claimed rewards      |
| `Revocation`        | `["revocation", reward_pool, user]`                           | Marker PDA blocking revoked users             |

## Instructions

| #   | Instruction                  | Signer         | Description                                              |
| --- | ---------------------------- | -------------- | -------------------------------------------------------- |
| 11  | `CreateContinuousPool`       | Authority      | Create pool with tracked/reward mints and balance source |
| 12  | `ContinuousOptIn`            | User           | Opt in; initial balance snapshot taken                   |
| 14  | `DistributeContinuousReward` | Authority      | Deposit rewards; accumulator updated                     |
| 16  | `SyncContinuousBalance`      | Permissionless | Sync user's on-chain token balance                       |
| 17  | `SetContinuousBalance`       | Authority      | Set user balance (AuthoritySet mode only)                |
| 15  | `ClaimContinuous`            | User           | Claim accrued rewards                                    |
| 20  | `SetContinuousMerkleRoot`    | Authority      | Set/rotate merkle root for cumulative claims             |
| 21  | `ClaimContinuousMerkle`      | User           | Claim via merkle proof over cumulative amount            |
| 19  | `RevokeContinuousUser`       | Authority      | Revoke user from pool                                    |
| 13  | `ContinuousOptOut`           | User           | Opt out and claim remaining rewards                      |
| 18  | `CloseContinuousPool`        | Authority      | Close pool, reclaim remaining tokens                     |

## Lifecycle

1. Authority calls `CreateContinuousPool` (creates pool PDA + reward vault ATA)
2. Users call `ContinuousOptIn` to start earning (creates UserRewardAccount, snapshots balance)
3. Authority calls `DistributeContinuousReward` to deposit reward tokens
4. Users call `ClaimContinuous` to withdraw accrued rewards
5. Balance syncs happen via `SyncContinuousBalance` (OnChain) or `SetContinuousBalance` (AuthoritySet)
6. Users call `ContinuousOptOut` to leave the pool and claim remaining rewards
7. Authority calls `CloseContinuousPool` after `clawback_ts` to reclaim remaining tokens

## Merkle Claim Mode

For high-scale or off-chain accounting use cases, an authority can publish cumulative snapshots:

1. Authority calls `SetContinuousMerkleRoot` with `(root, epoch)` (epoch must strictly increase)
2. Users claim with `ClaimContinuousMerkle` by providing `(epoch, cumulative_amount, proof)`
3. Program transfers `cumulative_amount - previously_claimed` (or a partial amount if requested)
4. Authority rotates root as needed (e.g. hourly or after each snapshot batch)

Once merkle mode is enabled (`epoch > 0`), accumulator-based withdrawal instructions (`ClaimContinuous`, `ContinuousOptOut`, `RevokeContinuousUser`) are blocked to avoid mixed accounting paths.

## Reward Accumulator

The program uses a reward-per-token accumulator to distribute rewards without iterating over all users:

1. **On distribute**: `delta_rpt = amount * PRECISION / opted_in_supply` is added to the pool's `reward_per_token`. An effective amount is back-computed to avoid vault dust from integer truncation.
2. **On settlement** (before any balance change, claim, opt-out, or revoke): `earned = user.last_known_balance * (pool.reward_per_token - user.reward_per_token_paid) / PRECISION` is added to the user's `accrued_rewards`.
3. **On balance sync**: After settling, `last_known_balance` is updated and the difference is applied to `opted_in_supply`.

`PRECISION = 1e12` ensures accurate accounting even with small individual balances.

## Revocation

If the pool was created with `revocable != 0`, the authority can call `RevokeContinuousUser`:

- **NonVested mode**: Accrued rewards are transferred to the user
- **Full mode**: Accrued rewards are returned to the authority

The UserRewardAccount is closed (rent refunded to `rent_destination`), and a `Revocation` marker PDA is created to permanently block re-opt-in.
