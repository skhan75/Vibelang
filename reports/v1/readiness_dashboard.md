# V1 Readiness Dashboard

Date: 2026-02-21

## Overall Status

- Release candidate cycle: `rc1-hosted-ci-validated`
- Blocking `P0` gates open: `1`
- `P1` gates open: `1`

## Gate Status Snapshot

| Gate | Status | Evidence | Owner | Notes |
| --- | --- | --- | --- | --- |
| Scope freeze | DONE | `docs/release/v1_scope_freeze.md` | Release | Local dry-run complete |
| Release gate mapping | DONE | `docs/release/v1_release_gates.md` | Release | Local dry-run complete |
| Blocker policy | DONE | `docs/release/release_blocker_policy.md` | Release | Local dry-run complete |
| Spec completeness | LOCAL-PASS | `reports/v1/spec_readiness.md`, workflow `v1-release-gates.yml` job `spec_integrity_gate` | Language/Release | Normative spec suite and blocking spec integrity gate are in place; runtime implementation parity for deferred surfaces tracked separately |
| Determinism | LOCAL-PASS | workflow `v1-release-gates.yml` job `determinism_and_core_smokes` | Compiler/CI | `phase2_native` + `frontend_fixtures` deterministic suites pass locally |
| Contract runtime enforcement | PARTIAL-P0 | `crates/vibe_cli/src/main.rs` preflight + `phase2_native` preflight tests | Compiler/Runtime | Dev/test contract example preflight is now enforced for build/run; full native runtime-path enforcement remains pending |
| Compiler self-host readiness | LOCAL-PASS | `reports/v1/selfhost_readiness.md`, workflow `v1-release-gates.yml` job `selfhost_readiness_gate`, `docs/development_checklist.md` section `7.3.e` | Compiler/Release | Local RC dry-run evidence recorded; 30-run CI counter tracking is active in self-host readiness report |
| Native dynamic containers (`Str`/`List`/`Map`) | LOCAL-PASS | `reports/v1/dynamic_containers_conformance.md`, workflow `v1-release-gates.yml` job `dynamic_containers_gate`, `docs/development_checklist.md` section `7.3.f.1` | Compiler/Runtime | `7.3.f.1` closeout surface implemented for v1 freeze scope with deterministic parser/type/runtime evidence |
| Ownership/sendability safety | LOCAL-PASS | `crates/vibe_cli/tests/phase7_v1_tightening.rs` + ownership fixtures | Compiler | Unknown sendability in `go` now fails closed (`E3201`) |
| Coverage thresholds | LOCAL-PASS | `docs/testing/coverage_policy.md`, `tooling/metrics/validate_phase7_coverage_matrix.py`, `tooling/metrics/validate_v1_quality_budgets.py` | QA/CI | Threshold validation wired into v1 gates workflow |
| Soak stability | LOCAL-PASS | workflow `v1-release-gates.yml` jobs `runtime_and_concurrency_smokes`, `quality_and_coverage_gate` | Runtime | Bounded runtime/concurrency smokes pass locally |
| Packaging integrity | LOCAL-PASS | workflow `v1-release-gates.yml` job `packaging_integrity_smoke` | Release/Tooling | Checksum generation gate added |
| Independent no-Cargo install | VALIDATED | `reports/v1/install_independence.md`, `reports/v1/phase8_ci_evidence.md`, workflow `v1-release-gates.yml` job `independent_install_gate`, workflow `v1-packaged-release.yml` jobs `install_smoke_linux`, `install_smoke_macos`, `install_smoke_windows` | Release/CI | Hosted cross-platform packaged install cycle is now validated for Phase 8 closure |
| Distribution trust stack (signature/provenance/SBOM) | VALIDATED | `reports/v1/distribution_readiness.md`, `reports/v1/phase8_ci_evidence.md`, workflow `v1-packaged-release.yml` jobs `packaged_reproducibility`, `sign_attest_and_sbom` | Release/Security/CI | Hosted signed/provenance/SBOM artifact cycle is now validated for Phase 8 closure |
| CLI help/version UX | VALIDATED | `docs/cli/help_manual.md`, `docs/cli/version_output.md`, tests `cli_help_snapshots`, `cli_version`, workflow `v1-cli-ux.yml`, `reports/v1/phase8_ci_evidence.md` | CLI/Docs | Regression-tested help/version UX is validated in hosted CI |
| Compatibility (upgrade/downgrade) | LOCAL-PASS | workflow `v1-release-gates.yml` job `compatibility_gate` | CLI/Release | Adjacent compatibility path currently represented by extension/lock-mode compatibility tests |
| Ops docs readiness | DONE | docs under `docs/release/`, `docs/support/`, `docs/privacy/` | Release | Required docs added |

## Open Exceptions

| ID | Severity | Owner | Mitigation | Due Date | Status |
| --- | --- | --- | --- | --- | --- |
| V1-P0-CRUNTIME | P0 | Compiler/Runtime | Build/run contract preflight landed; complete full native execution-path enforcement and add blocking smoke evidence | 2026-02-28 | Open |
| V1-P0-INSTALL-INDEPENDENCE | P0 | Release/CI | Complete first successful cross-platform `v1-packaged-release.yml` install smoke cycle and attach run evidence | 2026-03-05 | Closed |
| V1-P0-DISTRIBUTION-TRUST | P0 | Release/Security/CI | Complete first successful signed/provenance/SBOM artifact cycle and attach run evidence | 2026-03-05 | Closed |
| V1-P1-MEMTOOLS | P1 | Runtime/CI | Execute valgrind/GC feature-gated lanes in dedicated runner and attach artifacts | 2026-02-28 | Open |
| V1-P1-RCWORKFLOW | P1 | Release | Attach first successful `v1-release-gates.yml` run URL and artifacts to checklist/dashboard | 2026-02-24 | Closed |

## Required Report Links

- `reports/v1/release_candidate_checklist.md`
- `reports/v1/readiness_dashboard.md`
- `reports/v1/smoke_validation.md`
- `reports/v1/selfhost_readiness.md`
- `reports/v1/spec_readiness.md`
- `reports/v1/dynamic_containers_conformance.md`
- `reports/v1/install_independence.md`
- `reports/v1/distribution_readiness.md`
- `reports/v1/phase8_ci_evidence.md`
- `reports/v1/phase8_closeout_summary.md`
- Additional gate artifacts produced by `.github/workflows/v1-release-gates.yml`
