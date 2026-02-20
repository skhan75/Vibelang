# V1 Release Gates and Ownership

Date: 2026-02-20

This map converts remaining top-level unchecked guardrails into explicit v1 release
gates with ownership and evidence requirements.

## Gate Matrix

| Gate ID | Source Guardrail | Owner | Severity | Target Milestone | Required Evidence |
| --- | --- | --- | --- | --- | --- |
| VG-001 | Reproducible build mode (`--locked`, pinned toolchain, normalized artifacts) | CLI + Build | P0 | rc1 | Workflow pass + deterministic artifact report |
| VG-002 | Determinism tests (bit-identical output for same source/toolchain) | Compiler + CI | P0 | rc1 | `phase2_native` deterministic tests + v1 gate job |
| VG-003 | Memory safety defaults documented for user code paths | Language Docs + Runtime | P1 | rc1 | `docs/spec/memory_safety.md` + review signoff |
| VG-004 | Contract checks active in dev/test profiles by default | Compiler + Runtime | P0 | rc1 | Native run-path contract smoke tests |
| VG-005 | Unsafe escape hatch syntax and boundaries defined | Language + Compiler | P1 | rc2 | Spec + parser/checker tests + audit report output |
| VG-006 | Unsafe review path required | Compiler + Release | P1 | rc2 | Process doc + CI enforcement evidence |
| VG-007 | Unsafe block audit report emitted per build | CLI + Compiler | P1 | rc2 | `vibe build` audit artifact in CI |
| VG-008 | Allocation visibility in diagnostics/profile outputs | Compiler + Runtime | P1 | rc2 | Allocation visibility smoke report |
| VG-009 | Benchmark suite publishes CPU/memory/latency metrics per release | Runtime + Tooling | P1 | rc2 | `reports/v1/` benchmark artifacts |
| VG-010 | Cost model docs for copies/allocations/concurrency | Language Docs | P1 | rc2 | `docs/spec/cost_model.md` |
| VG-011 | Baseline compile benchmarks for clean/no-op/incremental | Tooling + CI | P1 | rc1 | Compile baseline report + validator pass |
| VG-012 | Incremental cache hit-rate telemetry in CI/local | Indexer + CLI | P1 | rc1 | Cache hit-rate metric and threshold gate |
| VG-013 | Compile latency regression thresholds configured | Tooling + CI | P1 | rc1 | Threshold validator in v1 gate workflow |

## Gate Policy

- `P0` gates block release candidate creation and promotion.
- `P1` gates may ship only with explicit exception signoff and dated follow-up.
- All gate exceptions must be recorded in `reports/v1/readiness_dashboard.md`.
