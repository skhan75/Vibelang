# M1 Self-Host Formatter Contract

Date: 2026-02-20

## Purpose

Freeze the M1 contract for self-host formatter readiness so CI can gate parity in
`Phase 7.3.e` with deterministic evidence.

## Scope (M1)

- Component: `selfhost/formatter_core.yb`
- Public function contract:
  - `pub format_source(src: Str) -> Str`
- Fixture corpus in scope:
  - `selfhost/fixtures/basic.input`
  - `selfhost/fixtures/nested.input`
- Expected outputs:
  - Byte-identical with host formatter outputs (`crates/vibe_fmt/src/lib.rs`)

Out of scope for M1:

- Full formatting coverage for all language constructs
- Promotion of self-host formatter as default production path
- M2+ components (docs/diagnostics formatter, compiler frontend slices)

## Determinism Requirements

For each fixture input:

1. host output == expected output
2. self-host output == expected output
3. host output == self-host output
4. repeated self-host execution is stable (same bytes across repeated runs)

No network/time/random dependencies are allowed in self-host conformance checks.

## Fallback Policy

- Host formatter remains authoritative/default.
- Self-host formatter executes in shadow/conformance mode only for M1.
- Any parity regression keeps host path active and blocks release promotion.

## RC Gate Conditions

M1 release gate passes when:

- self-host readiness CI job passes (`selfhost_readiness_gate`),
- parity and determinism evidence is published in
  `reports/v1/selfhost_readiness.md`,
- run counter is tracked toward 30 consecutive passing CI parity runs.

## Evidence Commands

- `cargo test -p vibe_fmt --test selfhost_conformance`
- `cargo run -q -p vibe_cli -- test selfhost/formatter_core.yb`
