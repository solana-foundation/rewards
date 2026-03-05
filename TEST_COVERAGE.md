# Test Coverage

## Estimated Coverage

> **Methodology**: This is a semantic coverage estimate produced by analyzing test
> assertions against the program's testable surface. It is not instrumented line
> coverage -- Solana SBF programs do not support LLVM coverage instrumentation.
> Coverage is risk-weighted: instruction handlers, account validation, and business
> logic errors carry more weight than events and utilities.

| Category                      | Covered | Total | Est. Coverage |
| ----------------------------- | ------- | ----- | ------------- |
| Instruction handlers          | 20      | 21    | 95%           |
| Account validation paths      | 72      | 84    | 86%           |
| Business logic error branches | 28      | 35    | 80%           |
| Custom error codes exercised  | 21      | 30    | 70%           |
| State & trait coverage (unit) | 25      | 27    | 93%           |
| Event & utility coverage      | 16      | 23    | 70%           |
| Security edge cases           | 4       | 5     | 80%           |
| **Overall (risk-weighted)**   |         |       | **~84%**      |

### Coverage Gaps

**Untested Instruction Handlers:**

- `EmitEvent`: No dedicated integration test (exercised indirectly via CPI from every other instruction)

**Untested Error Variants:**

- `InsufficientFunds` (5) -- no test explicitly asserts this error
- `MathOverflow` (7) -- no integration test triggers a math overflow condition on-chain
- `InvalidAccountData` (8) -- no test asserts `Custom(8)` (only `InstructionError::InvalidAccountData` is tested via `close_recipient` payer guard)
- `RentCalculationFailed` (10) -- no test triggers rent calculation failure
- `ClaimedAmountDecreased` (16) -- no integration test triggers this (unit tests cover it)
- `NoOptedInUsers` (20) -- no integration test distributes to pool with zero opted-in users
- `UserAlreadyOptedIn` (21) -- no integration test tries to double opt-in
- `UserNotOptedIn` (22) -- no integration test exercises this error
- `DistributionAmountTooSmall` (23) -- no test distributes too-small amount relative to supply

**Untested Validation Paths (partial list):**

- `SyncBalance`: no signer tests (permissionless instruction, no signers required)
- `SetBalance`: no event_authority, token_program, system_program checks (minimal validation instruction)
- `ClaimContinuous`: missing dedicated signer/writable/program validation tests (covered via continuous lifecycle but not isolated)
- `OptOut`: missing dedicated signer/writable/program validation tests (covered via continuous lifecycle but not isolated)
- `DistributeReward`: missing dedicated signer/writable/program validation tests
- `CloseRewardPool`: missing dedicated signer/writable/program validation tests
- `RevokeUser`: missing dedicated signer/writable/program validation tests

**Untested Processor Branches:**

- `DistributeReward`: `NoOptedInUsers` branch, `DistributionAmountTooSmall` branch
- `OptIn`: `UserRevoked` branch (revoke-then-opt-in scenario)
- `RevokeUser`: `UserAlreadyRevoked` branch, `DistributionNotRevocable` branch
- `SyncBalance`: `BalanceSourceMismatch` branch
- `SetBalance`: `BalanceSourceMismatch` branch
- `CloseRewardPool`: `ClawbackNotReached` branch

---

## Summary

| Instruction                   | File                                 | Test Count |
| ----------------------------- | ------------------------------------ | ---------- |
| CreateDirectDistribution      | `test_create_direct_distribution.rs` | 10         |
| AddDirectRecipient            | `test_add_direct_recipient.rs`       | 14         |
| ClaimDirect                   | `test_claim_direct.rs`               | 18         |
| CloseDirectDistribution       | `test_close_direct_distribution.rs`  | 13         |
| CloseDirectRecipient          | `test_close_direct_recipient.rs`     | 11         |
| CreateMerkleDistribution      | `test_create_merkle_distribution.rs` | 16         |
| ClaimMerkle                   | `test_claim_merkle.rs`               | 21         |
| CloseMerkleClaim              | `test_close_merkle_claim.rs`         | 9          |
| CloseMerkleDistribution       | `test_close_merkle_distribution.rs`  | 13         |
| RevokeDirectRecipient         | `test_revoke_direct_recipient.rs`    | 27         |
| RevokeMerkleClaim             | `test_revoke_merkle_claim.rs`        | 29         |
| Continuous (all 9 ixs)        | `test_continuous_lifecycle.rs`       | 31         |
| Cliff Vesting (cross-cutting) | `test_cliff_vesting.rs`              | 16         |
| **Total**                     |                                      | **228**    |

## Error Codes Validated

### System Errors (InstructionError)

- `MissingRequiredSignature` -- signer validation across all instructions
- `Immutable` -- writable account validation across all instructions
- `IncorrectProgramId` -- system/token/current program validation
- `InvalidInstructionData` -- empty and truncated data rejection
- `InvalidAccountOwner` -- wrong owner on program-owned accounts
- `InvalidAccountData` -- wrong payer on close_recipient
- `InvalidSeeds` -- PDA bump validation (via test helpers)

### Custom Errors (RewardsProgramError)

- `InvalidAmount` (0) -- zero amount in add_recipient, create_merkle_distribution
- `InvalidTimeWindow` (1) -- invalid vesting schedule in add_recipient
- `InvalidScheduleType` (2) -- tested via unit tests on VestingSchedule
- `UnauthorizedAuthority` (3) -- wrong authority on add_recipient, close_distribution, distribute, revoke
- `UnauthorizedRecipient` (4) -- wrong signer on claim_direct
- `NothingToClaim` (6) -- claim before vesting start, double-claim, idempotent claim
- `ExceedsClaimableAmount` (11) -- requesting more than vested in claim_direct, claim_merkle
- `InvalidMerkleProof` (12) -- wrong proof, wrong amount, wrong claimant
- `ClawbackNotReached` (13) -- close before clawback_ts
- `ClaimNotFullyVested` (14) -- close_recipient before fully claimed
- `InvalidCliffTimestamp` (15) -- cliff before start, cliff after end, cliff zero
- `DistributionNotRevocable` (17) -- revoke on non-revocable distribution, bitmask tests
- `InvalidRevokeMode` (18) -- invalid mode byte
- `ClaimantAlreadyRevoked` (19) -- double revoke, claim after revoke

---

## CreateDirectDistribution

**File:** `test_create_direct_distribution.rs`

### Error Tests

| Test                                                              | Description                   | Expected Error             |
| ----------------------------------------------------------------- | ----------------------------- | -------------------------- |
| `test_create_direct_distribution_missing_authority_signer`        | Authority not signed          | `MissingRequiredSignature` |
| `test_create_direct_distribution_missing_seeds_signer`            | Seeds keypair not signed      | `MissingRequiredSignature` |
| `test_create_direct_distribution_distribution_not_writable`       | Distribution PDA not writable | `Immutable`                |
| `test_create_direct_distribution_distribution_vault_not_writable` | Vault not writable            | `Immutable`                |
| `test_create_direct_distribution_wrong_system_program`            | Invalid system program        | `IncorrectProgramId`       |
| `test_create_direct_distribution_wrong_current_program`           | Invalid current program       | `IncorrectProgramId`       |
| `test_create_direct_distribution_empty_data`                      | Empty instruction data        | `InvalidInstructionData`   |
| `test_create_direct_distribution_truncated_data`                  | Truncated instruction data    | `InvalidInstructionData`   |

### Happy Path Tests

| Test                                                 | Description                                 |
| ---------------------------------------------------- | ------------------------------------------- |
| `test_create_direct_distribution_success`            | Creates distribution with correct PDA state |
| `test_create_direct_distribution_success_token_2022` | Creates distribution with Token-2022 mint   |

---

## AddDirectRecipient

**File:** `test_add_direct_recipient.rs`

### Error Tests

| Test                                                       | Description                | Expected Error             |
| ---------------------------------------------------------- | -------------------------- | -------------------------- |
| `test_add_direct_recipient_missing_authority_signer`       | Authority not signed       | `MissingRequiredSignature` |
| `test_add_direct_recipient_distribution_not_writable`      | Distribution not writable  | `Immutable`                |
| `test_add_direct_recipient_recipient_account_not_writable` | Recipient PDA not writable | `Immutable`                |
| `test_add_direct_recipient_wrong_system_program`           | Invalid system program     | `IncorrectProgramId`       |
| `test_add_direct_recipient_wrong_current_program`          | Invalid current program    | `IncorrectProgramId`       |
| `test_add_direct_recipient_empty_data`                     | Empty instruction data     | `InvalidInstructionData`   |
| `test_add_direct_recipient_truncated_data`                 | Truncated instruction data | `InvalidInstructionData`   |

### Custom Error Tests

| Test                                            | Description                          | Expected Error              |
| ----------------------------------------------- | ------------------------------------ | --------------------------- |
| `test_add_direct_recipient_unauthorized`        | Wrong authority key                  | `UnauthorizedAuthority` (3) |
| `test_add_direct_recipient_zero_amount`         | Zero token amount                    | `InvalidAmount` (0)         |
| `test_add_direct_recipient_invalid_time_window` | end_ts < start_ts                    | `InvalidTimeWindow` (1)     |
| `test_add_direct_recipient_insufficient_funds`  | Insufficient authority token balance | Token transfer CPI error    |

### Happy Path Tests

| Test                                           | Description                              |
| ---------------------------------------------- | ---------------------------------------- |
| `test_add_direct_recipient_success`            | Adds recipient with Linear vesting       |
| `test_add_direct_recipient_success_token_2022` | Adds recipient with Token-2022           |
| `test_add_direct_recipient_multiple`           | Adds two recipients to same distribution |

---

## ClaimDirect

**File:** `test_claim_direct.rs`

### Error Tests

| Test                                                     | Description                | Expected Error             |
| -------------------------------------------------------- | -------------------------- | -------------------------- |
| `test_claim_direct_missing_recipient_signer`             | Recipient not signed       | `MissingRequiredSignature` |
| `test_claim_direct_distribution_not_writable`            | Distribution not writable  | `Immutable`                |
| `test_claim_direct_recipient_account_not_writable`       | Recipient PDA not writable | `Immutable`                |
| `test_claim_direct_vault_not_writable`                   | Vault not writable         | `Immutable`                |
| `test_claim_direct_recipient_token_account_not_writable` | Token account not writable | `Immutable`                |
| `test_claim_direct_wrong_current_program`                | Invalid current program    | `IncorrectProgramId`       |
| `test_claim_direct_empty_data`                           | Empty instruction data     | `InvalidInstructionData`   |

### Custom Error Tests

| Test                                         | Description                   | Expected Error                |
| -------------------------------------------- | ----------------------------- | ----------------------------- |
| `test_claim_direct_nothing_before_start`     | Claim before vesting start    | `NothingToClaim` (6)          |
| `test_claim_direct_nothing_already_claimed`  | Double-claim after full claim | `NothingToClaim` (6)          |
| `test_claim_direct_unauthorized`             | Wrong signer claims           | `UnauthorizedRecipient` (4)   |
| `test_claim_direct_exceeds_claimable_amount` | Requesting more than vested   | `ExceedsClaimableAmount` (11) |

### Vesting Tests

| Test                                   | Description                       |
| -------------------------------------- | --------------------------------- |
| `test_claim_direct_partial_25_percent` | 25% linear vesting claim          |
| `test_claim_direct_partial_50_percent` | 50% linear vesting claim          |
| `test_claim_direct_multiple_claims`    | Sequential claims at 25% then 50% |
| `test_claim_direct_immediate_vesting`  | Immediate schedule full unlock    |

### Happy Path Tests

| Test                                   | Description                   |
| -------------------------------------- | ----------------------------- |
| `test_claim_direct_success_full`       | Full claim after vesting ends |
| `test_claim_direct_success_token_2022` | Full claim with Token-2022    |
| `test_claim_direct_specific_amount`    | Partial specific amount claim |

---

## CloseDirectDistribution

**File:** `test_close_direct_distribution.rs`

### Error Tests

| Test                                                                  | Description                          | Expected Error             |
| --------------------------------------------------------------------- | ------------------------------------ | -------------------------- |
| `test_close_direct_distribution_missing_authority_signer`             | Authority not signed                 | `MissingRequiredSignature` |
| `test_close_direct_distribution_distribution_not_writable`            | Distribution not writable            | `Immutable`                |
| `test_close_direct_distribution_distribution_vault_not_writable`      | Vault not writable                   | `Immutable`                |
| `test_close_direct_distribution_authority_token_account_not_writable` | Authority token account not writable | `Immutable`                |
| `test_close_direct_distribution_wrong_current_program`                | Invalid current program              | `IncorrectProgramId`       |
| `test_close_direct_distribution_empty_data`                           | Empty instruction data               | `InvalidInstructionData`   |

### Custom Error Tests

| Test                                          | Description     | Expected Error              |
| --------------------------------------------- | --------------- | --------------------------- |
| `test_close_direct_distribution_unauthorized` | Wrong authority | `UnauthorizedAuthority` (3) |

### Timelock Tests

| Test                                                                | Description              | Expected Error            |
| ------------------------------------------------------------------- | ------------------------ | ------------------------- |
| `test_close_direct_distribution_clawback_ts_before_timestamp_fails` | Close before clawback_ts | `ClawbackNotReached` (13) |

### Happy Path Tests

| Test                                                                  | Description                          |
| --------------------------------------------------------------------- | ------------------------------------ |
| `test_close_direct_distribution_success`                              | Closes distribution, PDA cleaned up  |
| `test_close_direct_distribution_success_token_2022`                   | Close with Token-2022                |
| `test_close_direct_distribution_returns_tokens`                       | Vault tokens returned to authority   |
| `test_close_direct_distribution_clawback_ts_zero_succeeds`            | clawback_ts=0 allows immediate close |
| `test_close_direct_distribution_clawback_ts_after_timestamp_succeeds` | Close after clawback_ts passes       |

---

## CloseDirectRecipient

**File:** `test_close_direct_recipient.rs`

### Error Tests

| Test                                                         | Description                 | Expected Error             |
| ------------------------------------------------------------ | --------------------------- | -------------------------- |
| `test_close_direct_recipient_missing_recipient_signer`       | Recipient not signed        | `MissingRequiredSignature` |
| `test_close_direct_recipient_original_payer_not_writable`    | Original payer not writable | `Immutable`                |
| `test_close_direct_recipient_recipient_account_not_writable` | Recipient PDA not writable  | `Immutable`                |
| `test_close_direct_recipient_wrong_current_program`          | Invalid current program     | `IncorrectProgramId`       |
| `test_close_direct_recipient_empty_data`                     | Empty instruction data      | `InvalidInstructionData`   |

### Custom Error Tests

| Test                                                 | Description                | Expected Error             |
| ---------------------------------------------------- | -------------------------- | -------------------------- |
| `test_close_direct_recipient_claim_not_fully_vested` | Close before fully claimed | `ClaimNotFullyVested` (14) |

### Security Tests

| Test                                               | Description         | Expected Error        |
| -------------------------------------------------- | ------------------- | --------------------- |
| `test_close_direct_recipient_wrong_original_payer` | Wrong payer address | `InvalidAccountData`  |
| `test_close_direct_recipient_wrong_recipient`      | Wrong recipient PDA | `InvalidAccountOwner` |

### Happy Path Tests

| Test                                             | Description                     |
| ------------------------------------------------ | ------------------------------- |
| `test_close_direct_recipient_success`            | Close fully-claimed recipient   |
| `test_close_direct_recipient_success_token_2022` | Close with Token-2022           |
| `test_close_direct_recipient_returns_rent`       | Rent returned to original payer |

---

## CreateMerkleDistribution

**File:** `test_create_merkle_distribution.rs`

### Error Tests

| Test                                                                   | Description                          | Expected Error             |
| ---------------------------------------------------------------------- | ------------------------------------ | -------------------------- |
| `test_create_merkle_distribution_missing_authority_signer`             | Authority not signed                 | `MissingRequiredSignature` |
| `test_create_merkle_distribution_missing_seeds_signer`                 | Seeds keypair not signed             | `MissingRequiredSignature` |
| `test_create_merkle_distribution_distribution_not_writable`            | Distribution not writable            | `Immutable`                |
| `test_create_merkle_distribution_distribution_vault_not_writable`      | Vault not writable                   | `Immutable`                |
| `test_create_merkle_distribution_authority_token_account_not_writable` | Authority token account not writable | `Immutable`                |
| `test_create_merkle_distribution_wrong_system_program`                 | Invalid system program               | `IncorrectProgramId`       |
| `test_create_merkle_distribution_wrong_current_program`                | Invalid current program              | `IncorrectProgramId`       |
| `test_create_merkle_distribution_empty_data`                           | Empty instruction data               | `InvalidInstructionData`   |
| `test_create_merkle_distribution_truncated_data`                       | Truncated instruction data           | `InvalidInstructionData`   |

### Custom Error Tests

| Test                                                | Description    | Expected Error      |
| --------------------------------------------------- | -------------- | ------------------- |
| `test_create_merkle_distribution_zero_amount`       | Fund amount=0  | `InvalidAmount` (0) |
| `test_create_merkle_distribution_zero_total_amount` | total_amount=0 | `InvalidAmount` (0) |

### Happy Path Tests

| Test                                                       | Description                           |
| ---------------------------------------------------------- | ------------------------------------- |
| `test_create_merkle_distribution_success`                  | Creates distribution with merkle root |
| `test_create_merkle_distribution_success_token_2022`       | Creates with Token-2022               |
| `test_create_merkle_distribution_funds_distribution_vault` | Verifies vault funded correctly       |
| `test_create_merkle_distribution_custom_merkle_root`       | Custom merkle root stored             |
| `test_create_merkle_distribution_custom_clawback`          | Custom clawback_ts stored             |

---

## ClaimMerkle

**File:** `test_claim_merkle.rs`

### Error Tests

| Test                                                    | Description                | Expected Error             |
| ------------------------------------------------------- | -------------------------- | -------------------------- |
| `test_claim_merkle_missing_claimant_signer`             | Claimant not signed        | `MissingRequiredSignature` |
| `test_claim_merkle_distribution_not_writable`           | Distribution not writable  | `Immutable`                |
| `test_claim_merkle_claim_account_not_writable`          | Claim PDA not writable     | `Immutable`                |
| `test_claim_merkle_vault_not_writable`                  | Vault not writable         | `Immutable`                |
| `test_claim_merkle_claimant_token_account_not_writable` | Token account not writable | `Immutable`                |
| `test_claim_merkle_wrong_system_program`                | Invalid system program     | `IncorrectProgramId`       |
| `test_claim_merkle_wrong_current_program`               | Invalid current program    | `IncorrectProgramId`       |

### Custom Error Tests

| Test                                               | Description                    | Expected Error                |
| -------------------------------------------------- | ------------------------------ | ----------------------------- |
| `test_claim_merkle_invalid_proof`                  | Corrupted proof bytes          | `InvalidMerkleProof` (12)     |
| `test_claim_merkle_wrong_amount_in_proof`          | Amount mismatch in proof       | `InvalidMerkleProof` (12)     |
| `test_claim_merkle_wrong_claimant`                 | Different claimant's proof     | `InvalidMerkleProof` (12)     |
| `test_claim_merkle_before_start`                   | Claim before linear start_ts   | `NothingToClaim` (6)          |
| `test_claim_merkle_amount_exceeds_available`       | Request exceeds vested         | `ExceedsClaimableAmount` (11) |
| `test_claim_merkle_idempotent_claim_creation`      | Second claim at same timestamp | `NothingToClaim` (6)          |
| `test_claim_merkle_reclaim_after_full_claim_fails` | Re-claim after full claim      | `NothingToClaim` (6)          |

### Vesting Tests

| Test                                        | Description              |
| ------------------------------------------- | ------------------------ |
| `test_claim_merkle_partial_claim_linear`    | 50% linear vesting claim |
| `test_claim_merkle_partial_claim_then_full` | Partial then full claim  |
| `test_claim_merkle_immediate_schedule`      | Immediate full unlock    |

### Happy Path Tests

| Test                                        | Description                               |
| ------------------------------------------- | ----------------------------------------- |
| `test_claim_merkle_success`                 | Full merkle claim with Immediate schedule |
| `test_claim_merkle_success_token_2022`      | Full claim with Token-2022                |
| `test_claim_merkle_specific_amount`         | Partial specific amount claim             |
| `test_claim_merkle_multiple_claimants_tree` | 4-claimant tree verification              |

---

## CloseMerkleClaim

**File:** `test_close_merkle_claim.rs`

### Error Tests

| Test                                                 | Description             | Expected Error             |
| ---------------------------------------------------- | ----------------------- | -------------------------- |
| `test_close_merkle_claim_missing_claimant_signer`    | Claimant not signed     | `MissingRequiredSignature` |
| `test_close_merkle_claim_claim_account_not_writable` | Claim PDA not writable  | `Immutable`                |
| `test_close_merkle_claim_wrong_current_program`      | Invalid current program | `IncorrectProgramId`       |
| `test_close_merkle_claim_empty_data`                 | Empty instruction data  | `InvalidInstructionData`   |
| `test_close_merkle_claim_distribution_not_closed`    | Distribution still open | `InvalidAccountOwner`      |

### Security Tests

| Test                                     | Description        | Expected Error        |
| ---------------------------------------- | ------------------ | --------------------- |
| `test_close_merkle_claim_wrong_claimant` | Wrong claimant PDA | `InvalidAccountOwner` |

### Happy Path Tests

| Test                                         | Description                           |
| -------------------------------------------- | ------------------------------------- |
| `test_close_merkle_claim_success`            | Close claim after distribution closed |
| `test_close_merkle_claim_success_token_2022` | Close with Token-2022                 |
| `test_close_merkle_claim_returns_rent`       | Rent returned to claimant             |

---

## CloseMerkleDistribution

**File:** `test_close_merkle_distribution.rs`

### Error Tests

| Test                                                                  | Description                          | Expected Error             |
| --------------------------------------------------------------------- | ------------------------------------ | -------------------------- |
| `test_close_merkle_distribution_missing_authority_signer`             | Authority not signed                 | `MissingRequiredSignature` |
| `test_close_merkle_distribution_distribution_not_writable`            | Distribution not writable            | `Immutable`                |
| `test_close_merkle_distribution_distribution_vault_not_writable`      | Vault not writable                   | `Immutable`                |
| `test_close_merkle_distribution_authority_token_account_not_writable` | Authority token account not writable | `Immutable`                |
| `test_close_merkle_distribution_wrong_current_program`                | Invalid current program              | `IncorrectProgramId`       |
| `test_close_merkle_distribution_empty_data`                           | Empty instruction data               | `InvalidInstructionData`   |

### Custom Error Tests

| Test                                             | Description              | Expected Error              |
| ------------------------------------------------ | ------------------------ | --------------------------- |
| `test_close_merkle_distribution_unauthorized`    | Wrong authority          | `UnauthorizedAuthority` (3) |
| `test_close_merkle_distribution_before_clawback` | Close before clawback_ts | `ClawbackNotReached` (13)   |

### Happy Path Tests

| Test                                                       | Description                         |
| ---------------------------------------------------------- | ----------------------------------- |
| `test_close_merkle_distribution_success`                   | Closes distribution, PDA cleaned up |
| `test_close_merkle_distribution_success_token_2022`        | Close with Token-2022               |
| `test_close_merkle_distribution_returns_tokens`            | Vault tokens returned to authority  |
| `test_close_merkle_distribution_returns_tokens_token_2022` | Token return with Token-2022        |
| `test_close_merkle_distribution_closes_distribution_vault` | Vault account closed                |

---

## RevokeDirectRecipient

**File:** `test_revoke_direct_recipient.rs`

### Error Tests

| Test                                               | Description                          | Expected Error             |
| -------------------------------------------------- | ------------------------------------ | -------------------------- |
| `test_revoke_missing_authority_signer`             | Authority not signed                 | `MissingRequiredSignature` |
| `test_revoke_distribution_not_writable`            | Distribution not writable            | `Immutable`                |
| `test_revoke_recipient_account_not_writable`       | Recipient PDA not writable           | `Immutable`                |
| `test_revoke_payer_not_writable`                   | Payer not writable                   | `Immutable`                |
| `test_revoke_vault_not_writable`                   | Vault not writable                   | `Immutable`                |
| `test_revoke_recipient_token_account_not_writable` | Recipient token account not writable | `Immutable`                |
| `test_revoke_wrong_current_program`                | Invalid current program              | `IncorrectProgramId`       |
| `test_revoke_empty_data`                           | Empty instruction data               | `InvalidInstructionData`   |

### Custom Error Tests

| Test                          | Description                              | Expected Error                  |
| ----------------------------- | ---------------------------------------- | ------------------------------- |
| `test_revoke_not_revocable`   | Distribution not revocable (revocable=0) | `DistributionNotRevocable` (17) |
| `test_revoke_wrong_authority` | Wrong authority key                      | `UnauthorizedAuthority` (3)     |
| `test_revoke_invalid_mode`    | Invalid revoke mode byte                 | `InvalidRevokeMode` (18)        |

### Bitmask Permission Tests

| Test                                                           | Description                   | Expected Error                  |
| -------------------------------------------------------------- | ----------------------------- | ------------------------------- |
| `test_revoke_non_vested_rejected_when_only_full_bit_set`       | revocable=2 rejects NonVested | `DistributionNotRevocable` (17) |
| `test_revoke_full_rejected_when_only_non_vested_bit_set`       | revocable=1 rejects Full      | `DistributionNotRevocable` (17) |
| `test_revoke_all_modes_rejected_when_revocable_0`              | revocable=0 rejects NonVested | `DistributionNotRevocable` (17) |
| `test_revoke_full_rejected_when_revocable_0`                   | revocable=0 rejects Full      | `DistributionNotRevocable` (17) |
| `test_revoke_both_modes_succeed_when_revocable_3`              | revocable=3 allows NonVested  | None                            |
| `test_revoke_full_succeeds_when_revocable_3`                   | revocable=3 allows Full       | None                            |
| `test_revoke_non_vested_succeeds_when_only_non_vested_bit_set` | revocable=1 allows NonVested  | None                            |
| `test_revoke_full_succeeds_when_only_full_bit_set`             | revocable=2 allows Full       | None                            |

### Vesting Tests

| Test                                  | Description                                   |
| ------------------------------------- | --------------------------------------------- |
| `test_revoke_before_vesting_starts`   | NonVested before start, all freed             |
| `test_revoke_after_full_vesting`      | NonVested after full vest, recipient gets all |
| `test_revoke_with_immediate_schedule` | Immediate schedule, all to recipient          |

### Happy Path Tests

| Test                                                | Description                             |
| --------------------------------------------------- | --------------------------------------- |
| `test_revoke_non_vested_at_midpoint`                | NonVested at 50%: split vested/unvested |
| `test_revoke_full_at_midpoint`                      | Full at 50%: authority reclaims all     |
| `test_revoke_with_token_2022`                       | NonVested at 50% with Token-2022        |
| `test_revoke_rent_returned_to_payer`                | Payer receives rent refund              |
| `test_revoke_freed_allocation_allows_new_recipient` | Freed allocation reusable               |

---

## RevokeMerkleClaim

**File:** `test_revoke_merkle_claim.rs`

### Error Tests

| Test                                                 | Description                 | Expected Error             |
| ---------------------------------------------------- | --------------------------- | -------------------------- |
| `test_revoke_merkle_missing_authority_signer`        | Authority not signed        | `MissingRequiredSignature` |
| `test_revoke_merkle_missing_payer_signer`            | Payer not signed            | `MissingRequiredSignature` |
| `test_revoke_merkle_distribution_not_writable`       | Distribution not writable   | `Immutable`                |
| `test_revoke_merkle_revocation_account_not_writable` | Revocation PDA not writable | `Immutable`                |
| `test_revoke_merkle_vault_not_writable`              | Vault not writable          | `Immutable`                |
| `test_revoke_merkle_claimant_token_not_writable`     | Claimant token not writable | `Immutable`                |
| `test_revoke_merkle_wrong_current_program`           | Invalid current program     | `IncorrectProgramId`       |
| `test_revoke_merkle_empty_data`                      | Empty instruction data      | `InvalidInstructionData`   |

### Custom Error Tests

| Test                                 | Description        | Expected Error                |
| ------------------------------------ | ------------------ | ----------------------------- |
| `test_revoke_merkle_wrong_authority` | Wrong authority    | `UnauthorizedAuthority` (3)   |
| `test_revoke_merkle_invalid_proof`   | Corrupted proof    | `InvalidMerkleProof` (12)     |
| `test_revoke_merkle_invalid_mode`    | Invalid mode byte  | `InvalidRevokeMode` (18)      |
| `test_revoke_merkle_double_revoke`   | Double revocation  | `ClaimantAlreadyRevoked` (19) |
| `test_claim_after_revocation_fails`  | Claim after revoke | `ClaimantAlreadyRevoked` (19) |

### Bitmask Permission Tests

| Test                                                                  | Description                   | Expected Error                  |
| --------------------------------------------------------------------- | ----------------------------- | ------------------------------- |
| `test_revoke_merkle_non_vested_rejected_when_only_full_bit_set`       | revocable=2 rejects NonVested | `DistributionNotRevocable` (17) |
| `test_revoke_merkle_full_rejected_when_only_non_vested_bit_set`       | revocable=1 rejects Full      | `DistributionNotRevocable` (17) |
| `test_revoke_merkle_all_modes_rejected_when_revocable_0`              | revocable=0 rejects NonVested | `DistributionNotRevocable` (17) |
| `test_revoke_merkle_full_rejected_when_revocable_0`                   | revocable=0 rejects Full      | `DistributionNotRevocable` (17) |
| `test_revoke_merkle_non_vested_succeeds_when_revocable_3`             | revocable=3 allows NonVested  | None                            |
| `test_revoke_merkle_full_succeeds_when_revocable_3`                   | revocable=3 allows Full       | None                            |
| `test_revoke_merkle_non_vested_succeeds_when_only_non_vested_bit_set` | revocable=1 allows NonVested  | None                            |
| `test_revoke_merkle_full_succeeds_when_only_full_bit_set`             | revocable=2 allows Full       | None                            |

### Happy Path Tests

| Test                                                 | Description                      |
| ---------------------------------------------------- | -------------------------------- |
| `test_revoke_merkle_non_vested_before_vesting_start` | NonVested before start           |
| `test_revoke_merkle_non_vested_at_midpoint`          | NonVested at 50%                 |
| `test_revoke_merkle_non_vested_after_full_vest`      | NonVested after full vest        |
| `test_revoke_merkle_full_at_midpoint`                | Full at 50%                      |
| `test_revoke_merkle_with_immediate_schedule`         | Immediate schedule               |
| `test_revoke_merkle_non_vested_after_partial_claim`  | NonVested after 25% claim        |
| `test_revoke_merkle_full_after_partial_claim`        | Full after 25% claim             |
| `test_revoke_merkle_with_token_2022`                 | NonVested at 50% with Token-2022 |

---

## Continuous Instructions

**File:** `test_continuous_lifecycle.rs`

This file covers all 9 continuous instructions (CreateRewardPool, OptIn, OptOut, DistributeReward, ClaimContinuous, SyncBalance, SetBalance, CloseRewardPool, RevokeUser) through lifecycle-based tests.

### CreateRewardPool -- Error Tests

| Test                                               | Description             | Expected Error             |
| -------------------------------------------------- | ----------------------- | -------------------------- |
| `test_create_reward_pool_missing_authority_signer` | Authority not signed    | `MissingRequiredSignature` |
| `test_create_reward_pool_missing_seeds_signer`     | Seeds not signed        | `MissingRequiredSignature` |
| `test_create_reward_pool_pool_not_writable`        | Pool not writable       | `Immutable`                |
| `test_create_reward_pool_wrong_system_program`     | Invalid system program  | `IncorrectProgramId`       |
| `test_create_reward_pool_wrong_current_program`    | Invalid current program | `IncorrectProgramId`       |
| `test_create_reward_pool_empty_data`               | Empty data              | `InvalidInstructionData`   |
| `test_create_reward_pool_truncated_data`           | Truncated data          | `InvalidInstructionData`   |

### CreateRewardPool -- Happy Path Tests

| Test                                         | Description                                   |
| -------------------------------------------- | --------------------------------------------- |
| `test_create_reward_pool_success`            | Creates pool with OnChain balance source      |
| `test_create_reward_pool_authority_set_mode` | Creates pool with AuthoritySet balance source |

### OptIn -- Error Tests

| Test                                   | Description                      | Expected Error             |
| -------------------------------------- | -------------------------------- | -------------------------- |
| `test_opt_in_missing_user_signer`      | User not signed                  | `MissingRequiredSignature` |
| `test_opt_in_pool_not_writable`        | Pool not writable                | `Immutable`                |
| `test_opt_in_user_reward_not_writable` | User reward account not writable | `Immutable`                |
| `test_opt_in_wrong_system_program`     | Invalid system program           | `IncorrectProgramId`       |
| `test_opt_in_wrong_current_program`    | Invalid current program          | `IncorrectProgramId`       |
| `test_opt_in_empty_data`               | Empty data                       | `InvalidInstructionData`   |

### OptIn -- Happy Path Tests

| Test                                          | Description                          |
| --------------------------------------------- | ------------------------------------ |
| `test_opt_in_success`                         | Opt in with on-chain tracked balance |
| `test_opt_in_authority_set_mode_zero_balance` | Opt in with AuthoritySet (balance=0) |

### DistributeReward -- Happy Path Tests

| Test                                           | Description                                   |
| ---------------------------------------------- | --------------------------------------------- |
| `test_distribute_reward_success`               | Distributes reward, verifies reward_per_token |
| `test_distribute_reward_updates_vault_balance` | Verifies vault balance increased              |

### ClaimContinuous -- Happy Path Tests

| Test                            | Description          |
| ------------------------------- | -------------------- |
| `test_claim_continuous_full`    | Full reward claim    |
| `test_claim_continuous_partial` | Partial reward claim |

### SyncBalance -- Happy Path Tests

| Test                                   | Description                                |
| -------------------------------------- | ------------------------------------------ |
| `test_sync_balance_increases_supply`   | Balance increase updates supply            |
| `test_sync_balance_decreases_supply`   | Balance decrease updates supply            |
| `test_sync_balance_after_distribution` | Sync settles rewards before balance update |

### SetBalance -- Happy Path Tests

| Test                       | Description                 |
| -------------------------- | --------------------------- |
| `test_set_balance_success` | Authority sets user balance |

### OptOut -- Happy Path Tests

| Test                        | Description                         |
| --------------------------- | ----------------------------------- |
| `test_opt_out_with_rewards` | Opt out auto-claims pending rewards |
| `test_opt_out_no_rewards`   | Opt out with zero rewards           |

### CloseRewardPool -- Happy Path Tests

| Test                                           | Description                           |
| ---------------------------------------------- | ------------------------------------- |
| `test_close_reward_pool_success`               | Closes empty pool                     |
| `test_close_reward_pool_with_remaining_tokens` | Closes pool, returns remaining tokens |

### Lifecycle (E2E) Tests

| Test                                   | Description                                                                   |
| -------------------------------------- | ----------------------------------------------------------------------------- |
| `test_full_lifecycle_on_chain_balance` | Full E2E: create -> 2 users opt in -> distribute -> claim -> opt out -> close |
| `test_lifecycle_authority_set_balance` | E2E with AuthoritySet mode                                                    |

---

## Cliff Vesting (Cross-cutting)

**File:** `test_cliff_vesting.rs`

### Direct Distribution -- Cliff Tests

| Test                                            | Description                                             | Expected Error       |
| ----------------------------------------------- | ------------------------------------------------------- | -------------------- |
| `test_cliff_direct_nothing_before_cliff`        | Nothing before cliff_ts                                 | `NothingToClaim` (6) |
| `test_cliff_direct_full_at_cliff`               | Full amount at cliff_ts                                 | None                 |
| `test_cliff_linear_direct_nothing_before_cliff` | CliffLinear nothing before cliff                        | `NothingToClaim` (6) |
| `test_cliff_linear_direct_accumulated_at_cliff` | CliffLinear ~25% at cliff                               | None                 |
| `test_cliff_linear_direct_50_percent`           | CliffLinear 50% at midpoint                             | None                 |
| `test_cliff_linear_direct_full_at_end`          | CliffLinear 100% at end                                 | None                 |
| `test_cliff_linear_direct_multiple_claims`      | CliffLinear claim at cliff then end                     | None                 |
| `test_cliff_linear_direct_account_state`        | Verifies PDA state after creating CliffLinear recipient | None                 |

### Merkle Distribution -- Cliff Tests

| Test                                            | Description                      | Expected Error       |
| ----------------------------------------------- | -------------------------------- | -------------------- |
| `test_cliff_linear_merkle_nothing_before_cliff` | CliffLinear nothing before cliff | `NothingToClaim` (6) |
| `test_cliff_linear_merkle_accumulated_at_cliff` | CliffLinear ~25% at cliff        | None                 |
| `test_cliff_linear_merkle_full_at_end`          | CliffLinear 100% at end          | None                 |
| `test_cliff_merkle_nothing_before_cliff`        | Cliff nothing before cliff       | `NothingToClaim` (6) |
| `test_cliff_merkle_full_at_cliff`               | Cliff 100% at cliff              | None                 |

### Validation Error Tests

| Test                                           | Description         | Expected Error               |
| ---------------------------------------------- | ------------------- | ---------------------------- |
| `test_cliff_linear_invalid_cliff_before_start` | cliff_ts < start_ts | `InvalidCliffTimestamp` (15) |
| `test_cliff_linear_invalid_cliff_after_end`    | cliff_ts > end_ts   | `InvalidCliffTimestamp` (15) |
| `test_cliff_schedule_zero_cliff_ts_invalid`    | cliff_ts=0          | `InvalidCliffTimestamp` (15) |

---

## Unit Tests

| Module               | File(s)                                                                                                                                                   | Test Count | Coverage                                                                                                                                                                            |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| state                | `direct_distribution.rs`, `direct_recipient.rs`, `merkle_claim.rs`, `merkle_distribution.rs`, `revocation.rs`, `reward_pool.rs`, `user_reward_account.rs` | 98         | Serialization round-trips, invalid data, PDA seeds, validation, trait accessors                                                                                                     |
| traits               | `account.rs`, `claim.rs`, `distribution.rs`, `event.rs`, `instruction.rs`, `pda.rs`, `vesting.rs`                                                         | 52         | Discriminators, deserialization edge cases, claim tracking, distribution validation, vesting calculations                                                                           |
| utils                | `balance_source.rs`, `claim_utils.rs`, `continuous_utils.rs`, `macros.rs`, `merkle_utils.rs`, `revoke_utils.rs`, `vesting_utils.rs`                       | 100        | Vesting schedule validation/serialization (45), merkle proof verification (14), continuous reward math (12), claim resolution (6), revoke mode (8), balance source (5), macros (10) |
| instructions         | `*/data.rs`, `emit_event/instruction.rs`                                                                                                                  | 60         | Instruction data parsing, length/discriminator validation, empty/truncated/valid data, VestingSchedule parsing                                                                      |
| events               | `claimed.rs`, `claim_closed.rs`, `distribution_closed.rs`, `distribution_created.rs`, `recipient_added.rs`, `recipient_revoked.rs`                        | 24         | Event serialization, discriminator bytes, to_bytes_inner layout                                                                                                                     |
| errors               | (via traits/instruction.rs)                                                                                                                               | 7          | Discriminator conversion, invalid discriminator handling                                                                                                                            |
| **Total Unit Tests** |                                                                                                                                                           | **341**    |                                                                                                                                                                                     |

---

## Test Categories

### 1. Signer Validation

Tests ensure all required signers are validated:

- Authority signer on create_distribution, add_recipient, close_distribution, distribute_reward, revoke_recipient, revoke_claim, create_pool, set_balance, revoke_user
- Recipient/claimant signer on claim_direct, close_recipient, claim_merkle, close_claim
- Payer signer on create_pool, opt_in, revoke_merkle_claim
- Seeds keypair signer on create_direct_distribution, create_merkle_distribution, create_pool
- User signer on opt_in, opt_out

### 2. Writable Account Validation

Tests ensure all writable accounts are enforced:

- Distribution/pool PDA writable on all mutation instructions
- Vault writable on claim, close, revoke instructions
- Token accounts writable on transfer instructions
- Recipient/claim PDAs writable on create/close/revoke
- Original payer writable on close_recipient, revoke

### 3. Program Validation

Tests verify correct program IDs:

- System program on create_distribution, create_pool, add_recipient, opt_in, claim_merkle
- Token program validated via ownership checks and ATA validation
- Current program on all instructions
- Event authority on all instructions (implicit via CPI)

### 4. PDA Validation

PDA derivation and bump validation covered by:

- Unit tests on all state struct PDA seeds
- Integration tests via generic `test_invalid_bump` helper
- Account validation in TryFrom implementations

### 5. Authority Authorization

Tests verify authority is the correct key:

- `UnauthorizedAuthority` tested on add_recipient, close_direct_distribution, close_merkle_distribution, revoke_direct_recipient, revoke_merkle_claim
- `UnauthorizedRecipient` tested on claim_direct

### 6. Token Operations

Token transfer and vault operations tested through:

- Successful claims transfer correct amounts (direct, merkle, continuous)
- Close distribution returns vault balance to authority
- Revoke distributes tokens per mode (NonVested vs Full)
- Token-2022 variant tests across all transfer operations
- Insufficient funds test for add_recipient

### 7. Vesting Schedules

Comprehensive vesting coverage:

- Immediate: full unlock at any time
- Linear: proportional unlock over [start_ts, end_ts]
- Cliff: full unlock at cliff_ts
- CliffLinear: linear unlock starting at cliff_ts
- Multiple sequential claims with cumulative balances
- 45 unit tests on VestingSchedule validation and calculation

### 8. Timelock / Clawback Enforcement

Clawback timestamp validation:

- `ClawbackNotReached` on close_direct_distribution and close_merkle_distribution before clawback_ts
- clawback_ts=0 allows immediate close
- Close succeeds after warping past clawback_ts

### 9. Revocation System

Revocation tested extensively:

- Bitmask permission tests (revocable=0,1,2,3 x NonVested/Full)
- NonVested mode: recipient gets vested, authority gets unvested
- Full mode: authority reclaims everything unclaimed
- Post-revocation claim blocked (`ClaimantAlreadyRevoked`)
- Double-revoke blocked (`ClaimantAlreadyRevoked`)
- Revoke after partial claim
- Rent returned to payer

### 10. Merkle Proof Verification

Merkle proof integrity:

- Invalid proof bytes rejected
- Wrong amount in proof rejected
- Wrong claimant cannot use another's proof
- 4-claimant tree verification
- 14 unit tests on proof computation and verification

---

**Grand Total: 341 unit tests + 228 integration tests = 569 total tests**
