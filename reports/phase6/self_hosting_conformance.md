# Phase 6.1 Self-Hosting Conformance Framework

Date: 2026-02-17

## Scope

This report defines the bootstrap-vs-selfhost conformance method and records
current evidence for the seeded formatter component.

## Framework

For each selfhost candidate component:

1. Define deterministic fixture inputs.
2. Capture expected selfhost output artifact(s).
3. Run host implementation against same fixture set.
4. Compare byte-for-byte output.
5. Repeat run to ensure deterministic stability.

Failure policy:

- Any mismatch blocks promotion.
- Any non-deterministic repeat output blocks promotion.

## Current Component: Formatter

- Selfhost prototype: `selfhost/formatter_core.yb`
- Fixture set:
  - `selfhost/fixtures/basic.input`
  - `selfhost/fixtures/nested.input`
- Expected outputs:
  - `selfhost/fixtures/basic.selfhost.out`
  - `selfhost/fixtures/nested.selfhost.out`
- Harness:
  - `crates/vibe_fmt/tests/selfhost_conformance.rs`

## Evidence Commands

- `cargo test -p vibe_fmt --test selfhost_conformance`

## Latest Result

- PASS: host formatter output matches selfhost fixture outputs.
- PASS: repeated formatting runs are deterministic for all fixtures.
