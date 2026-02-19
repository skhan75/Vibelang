# Phase 5 Baseline Freeze

Date: 2026-02-17

## Build and CI Baseline

- Phases 1-4 are marked complete in `docs/development_checklist.md`.
- Existing workflows:
  - `.github/workflows/phase1-frontend.yml`
  - `.github/workflows/phase2-native.yml`
  - `.github/workflows/phase3-concurrency.yml`
  - `.github/workflows/phase4-indexer-lsp.yml`
- Phase 5 workflow does not exist yet.

## Confirmed Phase 5 Gaps

- No sidecar crate in `crates/` and no executable sidecar service.
- No `vibe lint --intent` command in `crates/vibe_cli/src/main.rs`.
- No `reports/phase5/` evidence artifacts except this baseline record.
- No CI gating for phase5 sidecar/lint/budget thresholds.

## Core Conformance Blockers

- `while` and `repeat` are not lowered in native codegen (`crates/vibe_codegen/src/lib.rs`).
- `select` lowering currently executes only a subset (first-case path), not full multi-case semantics.
- `go` currently emits immediate call behavior (no detached task spawn in generated binaries).
- General `List/Map/member` lowering is incomplete in codegen.
- Runtime `@require/@ensure` checks are not active in native execution path by default.

## Phase 5 Implementation Start Point

Execution begins from this frozen baseline and proceeds through all Phase 5 workstreams:

1. Core conformance hardening.
2. Sidecar core implementation.
3. Intent lint and verifier gating.
4. Risk controls.
5. CI and evidence publication.
