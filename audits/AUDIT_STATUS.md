# Audit Status

Last updated: 2026-04-24

## Current Baseline

- Auditor: OtterSec
- Report: `audits/2026-ottersec-solana-foundation-rewards-audit.pdf`
- Audited-through commit: `aa1cfd9276375e44e57d1917d110ff095fb6d475`
- Compare unaudited delta: https://github.com/solana-foundation/rewards/compare/aa1cfd9276375e44e57d1917d110ff095fb6d475...main

Audit scope is commit-based. The audit was performed against commit `d795849` and all 8 findings were resolved and re-reviewed in PRs #32–#37 (last merged: PR #32, `aa1cfd9`). Commits after the audited-through SHA are considered unaudited until a new audit or mitigation review updates this file.

## Branch and Release Model

- `main` is the integration branch and may contain audited and unaudited commits.
- Stable production releases are immutable tags/releases (for example `v1.0.0`).
- Audited baselines are tracked by commit SHA plus immutable tags/releases, not by long-lived release branches.

## Verification Commands

```bash
# Count commits after the audited baseline
git rev-list --count aa1cfd9276375e44e57d1917d110ff095fb6d475..main

# Inspect commit list since audited baseline
git log --oneline aa1cfd9276375e44e57d1917d110ff095fb6d475..main

# Inspect file-level diff since audited baseline
git diff --name-status aa1cfd9276375e44e57d1917d110ff095fb6d475..main
```

## Maintenance Rules

When a new audit is completed:

1. Add the new report to `audits/`.
2. Update `Audited-through commit` and `Compare unaudited delta`.
3. Tag audited release commit(s) (for example `vX.Y.Z`).
4. Update README and release notes links if needed.
