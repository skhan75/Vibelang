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
| VG-009 | Benchmark suite publishes CPU/memory/latency metrics per release | Runtime + Tooling | P1 | rc2 | `reports/benchmarks/third_party/latest/results.json` + `reports/benchmarks/third_party/latest/summary.md` |
| VG-010 | Cost model docs for copies/allocations/concurrency | Language Docs | P1 | rc2 | `docs/spec/cost_model.md` |
| VG-011 | Baseline compile benchmarks for clean/no-op/incremental | Tooling + CI | P1 | rc1 | Hyperfine compile lanes in `reports/benchmarks/third_party/latest/results.json` + validator pass |
| VG-012 | Incremental cache hit-rate telemetry in CI/local | Indexer + CLI | P1 | rc1 | Cache hit-rate metric and threshold gate |
| VG-013 | Compile latency regression thresholds configured | Tooling + CI | P1 | rc1 | Threshold validator in v1 gate workflow |
| VG-014 | Independent install path validated on clean tier-1 machines (no Cargo dependency) | Release + CI + DX | P0 | rc2 | Packaged-install workflow pass + install independence report |
| VG-015 | Release artifacts are signed and attestable (checksums + signatures + provenance + SBOM) | Release + Security + CI | P0 | rc2 | Signing/provenance/SBOM workflow evidence + distribution readiness report |
| VG-016 | CLI discoverability maturity (`vibe --help`, `vibe --version`) is stable and regression-gated | CLI + Docs + CI | P1 | rc2 | CLI UX workflow pass + help/version docs/tests |
| VG-017 | Debug/profiling workflow defined for Vibe programs (symbols, stack traces, perf diagnostics) | Compiler + Runtime + DX | P1 | rc2 | `docs/debugging/workflow.md`, smoke tests, RC debug artifact |
| VG-018 | Runtime observability primitives contracted (structured logs/metrics/traces) | Runtime + Tooling | P1 | rc2 | `docs/observability/contracts.md`, observability smoke artifact |
| VG-019 | Production incident triage playbook for Vibe runtime failures | Runtime + Release | P1 | rc2 | `docs/support/production_incident_triage.md`, exercise report |
| VG-020 | Deterministic crash repro artifact format and collector | Compiler + Runtime + Tooling | P1 | rc2 | `docs/support/crash_repro_format.md`, collector tool + CI artifact |
| VG-021 | LTS/support windows and v1.x compatibility guarantees are explicit | Release + Docs | P1 | rc2 | `docs/support/lts_support_windows.md`, `docs/policy/compatibility_guarantees.md` |
| VG-022 | Security response/CVE handling workflow and disclosure policy | Security + Release | P0 | rc2 | `docs/security/cve_response_workflow.md`, `docs/security/disclosure_policy.md`, exercise evidence |
| VG-023 | Release-notes automation includes known limitations + breaking changes | Release + Tooling | P1 | rc2 | release-note generator + CI gate + RC artifact |
| VG-024 | Phase 7.4 docs/book program closed with executable snippet validation | Docs + DX + CI | P1 | rc2 | `book/`, docs CI gate, `reports/docs/documentation_quality.md` |
| VG-025 | Pilot application program evidence published (service + CLI/tooling) | Product + Runtime + DX | P0 | rc3 | pilot apps + metrics + case studies under `reports/phase14/` |
| VG-026 | GA promotion gate complete with consecutive hosted RC cycles and evidence bundle | Release + Security + CI | P0 | rc3 | hosted RC runs + GA readiness announcement + signed trust bundle |

## Gate Policy

- `P0` gates block release candidate creation and promotion.
- `P1` gates may ship only with explicit exception signoff and dated follow-up.
- All gate exceptions must be recorded in `reports/v1/readiness_dashboard.md`.

## Remaining-Work Mapping (Phase 13.2-14 + 7.4)

- `docs/development_checklist.md` `13.2.1` -> `VG-017`
- `docs/development_checklist.md` `13.2.2` -> `VG-018`
- `docs/development_checklist.md` `13.2.3` -> `VG-019`
- `docs/development_checklist.md` `13.2.4` -> `VG-020`
- `docs/development_checklist.md` `13.3.1` -> `VG-021`
- `docs/development_checklist.md` `13.3.2` -> `VG-022`
- `docs/development_checklist.md` `13.3.3` -> `VG-023`
- `docs/development_checklist.md` `13.3.4` and `7.4.*` -> `VG-024`
- `docs/development_checklist.md` `14.1.*` -> `VG-025`
- `docs/development_checklist.md` `14.2.*` plus Phase 13/14 exit criteria -> `VG-026`
