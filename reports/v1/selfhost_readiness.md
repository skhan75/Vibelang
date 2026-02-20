# V1 Self-Host Readiness Report

Date: 2026-02-20
Track: Phase 7.3.e Compiler Self-Host Readiness

## Scope

- M1 executable self-host formatter parity gate
- M3 starter shadow slice for deterministic diagnostics ordering parity
- Host fallback retained as default execution path

Contract reference:

- `docs/selfhost/m1_formatter_contract.md`

## Current Status

- Overall readiness state: `in-progress`
- M1 formatter parity gate: `local-pass`
- M3 shadow slice parity gate: `local-pass`
- Production default path: `host formatter`
- Self-host path mode: `shadow/conformance only`

## Evidence Commands (Latest Local Dry-Run)

- `cargo test -p vibe_fmt --test selfhost_conformance`
- `cargo run -q -p vibe_cli -- test selfhost/formatter_core.yb`
- `cargo test -p vibe_diagnostics --test selfhost_shadow_ordering`
- `cargo run -q -p vibe_cli -- test selfhost/diagnostics_ordering_shadow.yb`

## M1 Formatter Parity Metrics

| Metric | Value |
| --- | --- |
| Fixture corpus size | 2 |
| Host vs expected fixture parity | pass |
| Self-host executable example parity | pass |
| Repeat-run determinism (host formatter) | pass |
| Repeat-run determinism (self-host `vibe test` bridge) | pass |

## M3 Shadow Slice Metrics

| Metric | Value |
| --- | --- |
| Slice | deterministic diagnostics ordering contract |
| Host parity test | `crates/vibe_diagnostics/tests/selfhost_shadow_ordering.rs` pass |
| Self-host shadow executable examples | `selfhost/diagnostics_ordering_shadow.yb` pass |
| Default compiler path impact | none (shadow-only evidence) |

## Run Counter Toward M1 Exit

M1 exit requires 30 consecutive CI parity runs.

| Counter Field | Value |
| --- | --- |
| Consecutive passing runs required | 30 |
| Consecutive passing runs observed | 1 |
| Source of observed run | local dry-run |
| CI sequence tracking mode | active (to be advanced by `selfhost_readiness_gate` workflow runs) |

## Fallback and Safety Controls

- Host formatter remains the authoritative default path.
- Any parity regression in `selfhost_readiness_gate` blocks release promotion.
- Self-host artifacts are evidence-only until M4 transition gate.

## Go / No-Go Snapshot

- M1 gate for this run: `go` (local evidence complete)
- RC promotion from self-host perspective: `no-go yet` (requires CI-backed run streak toward 30, plus broader v1 blockers closure)
