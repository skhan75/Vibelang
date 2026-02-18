# Stdlib Stability Policy (Phase 2B)

This policy defines compatibility expectations for the Phase 2 standard library surface.

## Levels

- **stable**
  - backward-compatible within major version
  - behavior is deterministic and covered by tests
- **experimental**
  - may change in minor releases
  - requires explicit release-note callout
- **internal**
  - implementation detail, no compatibility guarantees

## Current Classification (Phase 2B)

- `io.print` / `io.println`: **stable**
- deterministic utility APIs used by `vibe test`: **experimental**
- runtime/internal shim symbols (`vibe_*` C symbols): **internal**

## Change Rules

- stable API signatures cannot break without major-version bump
- stable semantics changes require migration guidance
- experimental APIs may change, but must update:
  - docs
  - tests
  - release notes

## Determinism Requirement

All stable stdlib APIs must preserve deterministic behavior for identical inputs and toolchain versions.
