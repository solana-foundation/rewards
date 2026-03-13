# Compute Unit Benchmarks

This document tracks the compute unit (CU) consumption of each instruction in the Rewards Program.

## Running Benchmarks

To generate CU benchmarks, run:

```bash
just integration-test --with-cu
```

This runs all integration tests with `CU_TRACKING=1` enabled and updates the table below.

<!-- CU_SUMMARY_START -->

| Instruction               | Best  | Avg   | Worst | Count |
| ------------------------- | ----- | ----- | ----- | ----- |
| AddVestingRecipient       | 7453  | 9636  | 13533 | 20    |
| ClaimVesting              | 7589  | 12568 | 15242 | 7     |
| CloseVestingDistribution  | 10555 | 13598 | 18142 | 4     |
| CreateVestingDistribution | 18076 | 23628 | 31485 | 45    |

<!-- CU_SUMMARY_END -->

## Metrics

| Metric | Description                              |
| ------ | ---------------------------------------- |
| Best   | Lowest CU observed across all test runs  |
| Avg    | Average CU across all test runs          |
| Worst  | Highest CU observed across all test runs |
| Count  | Number of test invocations measured      |

## Notes

- CU values may vary slightly between runs due to account state differences
- The Solana runtime has a per-instruction limit of 200,000 CUs
- Lower CU consumption means lower transaction fees for users

## Confidential Transfer Overhead

When `pool.confidential_rewards != 0`, the following instructions incur additional CU cost
relative to the standard `TransferChecked` path:

| Instruction                  | Additional cost source                                              | Estimated overhead |
| ---------------------------- | ------------------------------------------------------------------- | ------------------ |
| `DistributeContinuousReward` | Extra `ConfidentialTransfer::Deposit` CPI after `TransferChecked`   | ~5k–10k CUs        |
| `ClaimContinuous`            | `ConfidentialTransfer::Transfer` replaces `TransferChecked`         | ~15k–25k CUs       |
| `ContinuousOptOut`           | Same as `ClaimContinuous` when accrued rewards > 0                  | ~15k–25k CUs       |

These estimates assume pre-verified proof context accounts (all instruction offsets = 0).
Actual values will be recorded here once integration tests run against a test validator
with the ZK ElGamal Proof program re-enabled (`--feature zk-elgamal-proof`).
