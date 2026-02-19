# Target Support Matrix

Date: 2026-02-17

## Tier Definitions

- **Tier 1**: full CI coverage (build, runtime smoke, determinism checks).
- **Tier 2**: build/conformance coverage, runtime smoke limited by host
  availability.
- **Tier 3**: planned/experimental only.

## Matrix (Phase 6)

| Target Triple | Tier | Build | Runtime Smoke | Determinism | Notes |
| --- | --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | Tier 1 | Yes | Yes | Yes | Primary CI baseline |
| `aarch64-unknown-linux-gnu` | Tier 2 | Yes (codegen+runtime compile path support) | Partial | Partial | Cross-runner availability dependent |
| `aarch64-apple-darwin` | Tier 2 | Yes (codegen+runtime compile path support) | Partial | Partial | Requires macOS runner for full smoke |

## Governance Rules

- Tier status must match CI evidence.
- Any regression must be logged in `docs/targets/limitations_register.md`.
- Phase exit criteria require documented evidence links per tier.
