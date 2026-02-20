# V1 Scope Freeze

Date: 2026-02-20

## Purpose

This document locks what is considered v1 release scope, what is explicitly out of
scope, and what is deferred to post-v1.

Reference alignment:

- `docs/charter.md`
- `docs/non_goals.md`
- `docs/development_checklist.md` (Phase 7.3)

## In Scope for v1

- Deterministic native compile and run path (`vibe check/build/run/test`)
- Contracts and intent annotations as stable language features:
  - `@intent`, `@examples`, `@require`, `@ensure`, `@effect`
- Concurrency primitives with safety diagnostics:
  - `go`, `chan`, `select`, `after`
- Formatter, docs generator, package manager baseline:
  - `vibe fmt`, `vibe doc`, `vibe pkg`
- Indexer and LSP baseline for references/diagnostics
- Optional AI intent linting that remains non-blocking and outside compile correctness
- Release governance baseline:
  - scope freeze, blocker policy, RC process, rollback playbook, limitations gate

## Out of Scope for v1

- Mandatory cloud AI for compilation or correctness
- Hard real-time runtime guarantees
- Full Rust-style borrow checker/lifetimes
- Public stable plugin ABI commitments beyond current documented tooling surfaces
- Broad domain expansion beyond backend/concurrent/native-first workloads

## Deferred to Post-v1

- Full self-hosted toolchain transition
- Advanced AI code generation/autocomplete quality guarantees
- Expanded target matrix requiring multi-host runtime smoke parity
- Extended GC observability guarantees as a release blocker (until GC path is active in runtime)
- Rich package registry trust stack (provenance/signing automation maturity beyond v1 baseline)

## Freeze Change Policy

- Any change to v1 scope requires:
  1. Written rationale in release PR description
  2. Impact statement on compatibility and timelines
  3. Approval from release owner + compiler owner
  4. Checklist update in `docs/development_checklist.md`
