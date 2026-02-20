# V1 Readiness Dashboard

Date: 2026-02-20

## Overall Status

- Release candidate cycle: `rc1-dryrun-local`
- Blocking `P0` gates open: `2`
- `P1` gates open: `2`

## Gate Status Snapshot

| Gate | Status | Evidence | Owner | Notes |
| --- | --- | --- | --- | --- |
| Scope freeze | DONE | `docs/release/v1_scope_freeze.md` | Release | Local dry-run complete |
| Release gate mapping | DONE | `docs/release/v1_release_gates.md` | Release | Local dry-run complete |
| Blocker policy | DONE | `docs/release/release_blocker_policy.md` | Release | Local dry-run complete |
| Determinism | LOCAL-PASS | workflow `v1-release-gates.yml` job `determinism_and_core_smokes` | Compiler/CI | `phase2_native` + `frontend_fixtures` deterministic suites pass locally |
| Contract runtime enforcement | PARTIAL-P0 | `crates/vibe_cli/src/main.rs` preflight + `phase2_native` preflight tests | Compiler/Runtime | Dev/test contract example preflight is now enforced for build/run; full native runtime-path enforcement remains pending |
| Compiler self-host readiness | LOCAL-PASS | `reports/v1/selfhost_readiness.md`, workflow `v1-release-gates.yml` job `selfhost_readiness_gate`, `docs/development_checklist.md` section `7.3.e` | Compiler/Release | Local RC dry-run evidence recorded; 30-run CI counter tracking is active in self-host readiness report |
| Native dynamic containers (`Str`/`List`/`Map`) | OPEN-P0 | `docs/development_checklist.md` section `7.3.f Language Surface + Dynamic Runtime Data Structures (Second, After 7.3.e)` | Compiler/Runtime | Native backend still contains release-critical unsupported fallbacks (`E3401`/`E3402`) for dynamic container/member paths |
| Ownership/sendability safety | LOCAL-PASS | `crates/vibe_cli/tests/phase7_v1_tightening.rs` + ownership fixtures | Compiler | Unknown sendability in `go` now fails closed (`E3201`) |
| Coverage thresholds | LOCAL-PASS | `docs/testing/coverage_policy.md`, `tooling/metrics/validate_phase7_coverage_matrix.py`, `tooling/metrics/validate_v1_quality_budgets.py` | QA/CI | Threshold validation wired into v1 gates workflow |
| Soak stability | LOCAL-PASS | workflow `v1-release-gates.yml` jobs `runtime_and_concurrency_smokes`, `quality_and_coverage_gate` | Runtime | Bounded runtime/concurrency smokes pass locally |
| Packaging integrity | LOCAL-PASS | workflow `v1-release-gates.yml` job `packaging_integrity_smoke` | Release/Tooling | Checksum generation gate added |
| Compatibility (upgrade/downgrade) | LOCAL-PASS | workflow `v1-release-gates.yml` job `compatibility_gate` | CLI/Release | Adjacent compatibility path currently represented by extension/lock-mode compatibility tests |
| Ops docs readiness | DONE | docs under `docs/release/`, `docs/support/`, `docs/privacy/` | Release | Required docs added |

## Open Exceptions

| ID | Severity | Owner | Mitigation | Due Date | Status |
| --- | --- | --- | --- | --- | --- |
| V1-P0-CRUNTIME | P0 | Compiler/Runtime | Build/run contract preflight landed; complete full native execution-path enforcement and add blocking smoke evidence | 2026-02-28 | Open |
| V1-P0-DYNCONTAINERS | P0 | Compiler/Runtime | Implement native dynamic container lowering (`Str`/`List`/`Map`) and publish conformance evidence for algorithmic container-heavy fixtures after `V1-P0-SELFHOSTREADY` gate | 2026-03-12 | Open |
| V1-P1-MEMTOOLS | P1 | Runtime/CI | Execute valgrind/GC feature-gated lanes in dedicated runner and attach artifacts | 2026-02-28 | Open |
| V1-P1-RCWORKFLOW | P1 | Release | Attach first successful `v1-release-gates.yml` run URL and artifacts to checklist/dashboard | 2026-02-24 | Open |

## Required Report Links

- `reports/v1/release_candidate_checklist.md`
- `reports/v1/readiness_dashboard.md`
- `reports/v1/smoke_validation.md`
- `reports/v1/selfhost_readiness.md`
- Additional gate artifacts produced by `.github/workflows/v1-release-gates.yml`
