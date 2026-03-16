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
