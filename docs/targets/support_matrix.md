# Target Support Matrix

Date: 2026-02-17

## Tier Definitions

- **Tier 1**: full CI coverage (build, runtime smoke, determinism checks).
- **Tier 2**: build/conformance coverage, runtime smoke limited by host
  availability.
- **Tier 3**: planned/experimental only.

## Matrix (Phase 8 Full-Tier1 Independence Target)

| Target Triple | Tier | Build | Runtime Smoke | Determinism | Notes |
| --- | --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | Tier 1 | Yes | Yes | Yes | Tier-1 packaged install candidate |
| `x86_64-apple-darwin` | Tier 1 | Yes | Yes | Yes | Tier-1 packaged install candidate |
| `x86_64-pc-windows-msvc` | Tier 1 | Yes | Yes | Yes | Tier-1 packaged install candidate |
| `aarch64-unknown-linux-gnu` | Tier 2 | Yes (codegen+runtime compile path support) | Partial | Partial | Cross-runner availability dependent |
| `aarch64-apple-darwin` | Tier 2 | Yes (codegen+runtime compile path support) | Partial | Partial | Arm64 parity remains a tracked expansion |

## Governance Rules

- Tier status must match CI evidence.
- Any regression must be logged in `docs/targets/limitations_register.md`.
- Phase exit criteria require documented evidence links per tier.
- Tier-1 packaged release status additionally requires clean-machine install evidence
  and signed artifact trust evidence (checksum + signature + provenance + SBOM).
