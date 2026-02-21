# V1 Selfhost M2 Readiness Report

Date: 2026-02-21

## Objective

Track M2 tooling-component readiness for self-host parity without changing
default production execution paths.

## Scope

- Docs formatter shadow component:
  - `selfhost/docs_formatter_core.yb`
  - host parity target: `crates/vibe_doc/src/lib.rs`
- Diagnostics formatter shadow component:
  - `selfhost/diagnostics_formatter_core.yb`
  - host parity target: `crates/vibe_diagnostics/src/lib.rs`
- Contract doc:
  - `docs/selfhost/m2_formatter_diagnostics_contract.md`

## Fixture Parity Matrix

| Component | Fixture(s) | Host Parity Harness | Status |
| --- | --- | --- | --- |
| Docs formatter | `docs_basic`, `docs_multi` | `cargo test -p vibe_doc --test selfhost_conformance` | validated |
| Diagnostics formatter | `diagnostics_basic`, `diagnostics_severity` | `cargo test -p vibe_diagnostics --test selfhost_formatter_conformance` | validated |

## Determinism and Contract Execution

- Docs formatter self-host contract execution:
  - `cargo run -q -p vibe_cli -- test selfhost/docs_formatter_core.yb`
- Diagnostics formatter self-host contract execution:
  - `cargo run -q -p vibe_cli -- test selfhost/diagnostics_formatter_core.yb`
- Repeat-run determinism checks are enforced in both parity harnesses:
  - `selfhost_docs_formatter_repeat_runs_are_deterministic`
  - `selfhost_diagnostics_formatter_repeat_runs_are_deterministic`

## CI Gate Wiring

- Blocking release gate job:
  - `.github/workflows/v1-release-gates.yml` job `selfhost_m2_gate`
- Gate verifies:
  - host-vs-selfhost parity harnesses for M2 fixtures
  - self-host contract execution via `vibe test`
  - presence of M2 contract and readiness documents

## Readiness Status

- Contract definition: `complete`
- Fixture parity: `validated`
- Determinism checks: `validated`
- Release-gate integration: `complete`

## Open Drift / Follow-Ups

- None in M2 scope at this time; broader self-host expansion continues in M3/M4.
