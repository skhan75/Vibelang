# Phase 6.1 Self-Hosting Milestones

Date: 2026-02-17

## Milestone Plan

## M0: Seed and Validate Prototype (Done)

- Seeded `selfhost/formatter_core.yb`.
- Added fixture corpus under `selfhost/fixtures/`.
- Added host-vs-selfhost conformance checks and repeat-run determinism tests.

Exit signal:

- `cargo test -p vibe_fmt --test selfhost_conformance` passes.

## M1: Executable Selfhost Formatter

- Implement parser + formatter core in VibeLang for fixture corpus.
- Produce output artifacts from selfhost implementation in CI.
- Keep host formatter as fallback.

Exit signal:

- Host and selfhost formatter outputs match across all fixtures for 30
  consecutive CI runs.

## M2: Selfhost Docs/Diagnostics Formatter

- Port docs extractor/diagnostic text formatter to VibeLang.
- Reuse same parity harness shape as formatter.

Exit signal:

- Stable parity and deterministic run evidence for docs/diagnostic component.

## M3: Compiler Frontend Slice

- Port one deterministic frontend slice (e.g. AST pretty printer or diagnostics
  sorting pass).
- Integrate as optional CI shadow execution path.

Exit signal:

- Output equivalence proof and no performance regression above agreed budget.

## M4: Selfhost Transition Gate

- Publish graduation criteria for replacing host component in default path.
- Include rollback toggle and release notes migration steps.
