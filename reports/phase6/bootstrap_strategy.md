# Phase 6.1 Bootstrap Strategy

Date: 2026-02-17

## Goal

Move from host-implemented toolchain components to VibeLang-owned
implementations without breaking deterministic behavior.

## Strategy

1. Keep the host implementation as source of truth for runtime behavior.
2. Build VibeLang-owned tooling components behind conformance fixtures.
3. Require host-vs-selfhost output parity before switching execution paths.
4. Promote one component at a time (formatter first, then docs/diagnostics).

## Transition Phases

- **Bootstrap phase (current)**:
  - Host runs all production logic.
  - Selfhost component exists as executable design/prototype + fixture outputs.
- **Dual-run phase**:
  - CI runs host + selfhost component on same fixtures.
  - Gate on byte-equal outputs for deterministic tools.
- **Promotion phase**:
  - Selfhost component can be selected behind explicit flag.
  - Host implementation remains fallback until parity is sustained.

## Determinism Guardrails

- Exact output matching on canonical fixtures.
- Repeat-run stability checks (`run N times`, same bytes each run).
- No network/time/random dependencies in conformance fixtures.

## Initial Component

`selfhost/formatter_core.yb` is the first seeded component.

Host parity is validated by `crates/vibe_fmt/tests/selfhost_conformance.rs`.
