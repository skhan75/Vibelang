# M2 Formatter + Diagnostics Contract

Date: 2026-02-21

## Purpose

Define the M2 self-host scope for tooling components that can be parity-tested
without promoting default compiler/runtime execution paths.

## Components in Scope

- Docs formatter shadow component: `selfhost/docs_formatter_core.yb`
- Diagnostics formatter shadow component:
  `selfhost/diagnostics_formatter_core.yb`

## Host Implementations

- Docs formatter host path: `crates/vibe_doc/src/lib.rs`
  - `extract_docs()`
  - `render_markdown()`
- Diagnostics formatter host path: `crates/vibe_diagnostics/src/lib.rs`
  - `Diagnostics::to_golden()`
  - deterministic ordering contract in `Diagnostics::into_sorted()`

## Inputs and Outputs

### Docs Formatter

- Input: Vibe source text fixture (`*.input`)
- Output: rendered markdown (`*.selfhost.out`)
- Parity requirement: host-rendered markdown must match the self-host fixture
  output byte-for-byte.

### Diagnostics Formatter

- Input: deterministic diagnostics fixture key (`*.input`)
- Output: golden diagnostic text (`*.selfhost.out`)
- Parity requirement: host golden output must match self-host fixture output
  byte-for-byte.

## Determinism Requirements

- Re-running host formatter logic with identical inputs must emit identical
  output bytes.
- Re-running `vibe test` for M2 self-host contracts must not produce semantic
  drift (example pass/fail counts are stable).

## CI/Test Coverage

- `cargo test -p vibe_doc --test selfhost_conformance`
- `cargo test -p vibe_diagnostics --test selfhost_formatter_conformance`
- `cargo run -q -p vibe_cli -- test selfhost/docs_formatter_core.yb`
- `cargo run -q -p vibe_cli -- test selfhost/diagnostics_formatter_core.yb`

## Out of Scope (M2)

- Switching production defaults to self-host implementations
- Runtime/compiler core self-host promotion (M3/M4 scope)
