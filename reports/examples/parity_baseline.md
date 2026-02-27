# Examples Parity Baseline

Generated at (UTC): `2026-02-27T07:47:06Z`

## Summary

- Total examples: `75`
- `vibe check`: `75` pass / `0` fail
- `vibe run`: `46` pass / `29` fail

## Intentional Failure Demos (allowlisted)

- `examples/10_contracts_intent/68_runtime_require_failure_demo.yb`
- `examples/10_contracts_intent/69_runtime_ensure_failure_demo.yb`

## Failure Inventory Source

Detailed failing-file inventory and first-failure output are recorded in:

- `reports/examples/parity_baseline.json`

## Notes

- This baseline is the execution starting point for checklist items `A-01..A-08`.
- Any example failure not covered by the allowlist must map to a checklist ID in
  `docs/checklists/features_and_optimizations.md`.
