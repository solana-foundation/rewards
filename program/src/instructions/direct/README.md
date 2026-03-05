# Direct Distribution

Creates on-chain accounts per recipient with individual vesting schedules.

## How It Works

The authority creates a `DirectDistribution` PDA, then adds recipients one by one via `AddDirectRecipient`. Each recipient gets their own `DirectRecipient` PDA storing their allocation amount and vesting schedule. Tokens are transferred from the authority to the distribution vault as recipients are added.

Recipients claim vested tokens via `ClaimDirect`. The amount claimable depends on the vesting schedule (Immediate, Linear, Cliff, or CliffLinear).

## Account Structure

| Account              | PDA Seeds                                        | Description                                   |
| -------------------- | ------------------------------------------------ | --------------------------------------------- |
| `DirectDistribution` | `["direct_distribution", mint, authority, seed]` | Distribution config (authority, mint, totals) |
| `DirectRecipient`    | `["direct_recipient", distribution, recipient]`  | Per-recipient allocation and vesting          |
| `Revocation`         | `["revocation", distribution, recipient]`        | Marker PDA blocking revoked recipients        |

## Instructions

| #   | Instruction                | Signer    | Description                                         |
| --- | -------------------------- | --------- | --------------------------------------------------- |
| 0   | `CreateDirectDistribution` | Authority | Create distribution PDA and vault ATA               |
| 1   | `AddDirectRecipient`       | Authority | Add recipient with amount and vesting schedule      |
| 2   | `ClaimDirect`              | Recipient | Claim vested tokens                                 |
| 9   | `RevokeDirectRecipient`    | Authority | Revoke a recipient (NonVested or Full mode)         |
| 4   | `CloseDirectRecipient`     | Recipient | Close recipient PDA, reclaim rent                   |
| 3   | `CloseDirectDistribution`  | Authority | Close distribution after clawback_ts, reclaim funds |

## Lifecycle

1. Authority calls `CreateDirectDistribution` (creates PDA + vault ATA)
2. Authority calls `AddDirectRecipient` for each user (transfers tokens to vault)
3. Recipients call `ClaimDirect` to withdraw vested tokens over time
4. After full vesting, recipients call `CloseDirectRecipient` to reclaim rent
5. Authority calls `CloseDirectDistribution` after `clawback_ts` to reclaim remaining tokens

## Revocation

If the distribution was created with `revocable != 0`, the authority can call `RevokeDirectRecipient`:

- **NonVested mode**: Vested-but-unclaimed tokens go to recipient; unvested tokens freed to vault
- **Full mode**: All unclaimed tokens returned to authority
