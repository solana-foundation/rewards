# Merkle Distribution

Stores a single merkle root on-chain; recipients provide proofs to claim their allocations.

## How It Works

The authority builds a merkle tree off-chain where each leaf encodes `(claimant, total_amount, vesting_schedule)`. The root is submitted via `CreateMerkleDistribution`, which also funds the distribution vault. Recipients claim by providing their leaf data and a merkle proof via `ClaimMerkle`. The program verifies the proof against the on-chain root and transfers vested tokens.

No per-recipient accounts are created until someone actually claims, making this model scale to millions of recipients with constant on-chain storage.

## Account Structure

| Account              | PDA Seeds                                        | Description                           |
| -------------------- | ------------------------------------------------ | ------------------------------------- |
| `MerkleDistribution` | `["merkle_distribution", mint, authority, seed]` | Distribution config with merkle root  |
| `MerkleClaim`        | `["merkle_claim", distribution, claimant]`       | Tracks claimed amount per claimant    |
| `Revocation`         | `["revocation", distribution, claimant]`         | Marker PDA blocking revoked claimants |

## Instructions

| #   | Instruction                | Signer    | Description                                                |
| --- | -------------------------- | --------- | ---------------------------------------------------------- |
| 5   | `CreateMerkleDistribution` | Authority | Create distribution with merkle root, fund vault           |
| 6   | `ClaimMerkle`              | Claimant  | Prove allocation via merkle proof, claim vested tokens     |
| 10  | `RevokeMerkleClaim`        | Authority | Revoke a claimant (NonVested or Full mode)                 |
| 7   | `CloseMerkleClaim`         | Claimant  | Close claim PDA after distribution is closed, reclaim rent |
| 8   | `CloseMerkleDistribution`  | Authority | Close distribution after clawback_ts, reclaim funds        |

## Lifecycle

1. Authority builds merkle tree off-chain from recipient list
2. Authority calls `CreateMerkleDistribution` (submits root, funds vault)
3. Recipients call `ClaimMerkle` with their proof to withdraw vested tokens over time
4. Authority calls `CloseMerkleDistribution` after `clawback_ts` to reclaim remaining tokens
5. Claimants call `CloseMerkleClaim` after closure to reclaim rent

## Claiming

Each `ClaimMerkle` call:

1. Verifies the merkle proof against the on-chain root
2. Checks the revocation marker PDA to ensure the claimant is not revoked
3. Computes the vested amount from the `VestingSchedule`, subtracts already-claimed tokens
4. Transfers available tokens from the vault to the claimant

Pass `amount = 0` to claim all currently vested tokens.

## Revocation

If the distribution was created with `revocable != 0`, the authority can call `RevokeMerkleClaim`:

- **NonVested mode**: Vested-but-unclaimed tokens are transferred to the claimant; unvested tokens are freed back to the vault
- **Full mode**: All unclaimed tokens (vested and unvested) are returned to the authority

A `Revocation` marker PDA is created to permanently block future claims. Revocation works even if the claimant has never claimed (claimed_amount defaults to 0).
