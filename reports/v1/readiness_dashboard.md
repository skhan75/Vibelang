# V1 Readiness Dashboard

Date: 2026-02-21

## Overall Status

- Release candidate cycle: `rc1-phase11-module-composition`
- Blocking `P0` gates open: `0`
- `P1` gates open: `0`

## Gate Status Snapshot

| Gate | Status | Evidence | Owner | Notes |
| --- | --- | --- | --- | --- |
| Scope freeze | DONE | `docs/release/v1_scope_freeze.md` | Release | Local dry-run complete |
| Release gate mapping | DONE | `docs/release/v1_release_gates.md` | Release | Local dry-run complete |
| Blocker policy | DONE | `docs/release/release_blocker_policy.md` | Release | Local dry-run complete |
| Spec completeness | LOCAL-PASS | `reports/v1/spec_readiness.md`, workflow `v1-release-gates.yml` job `spec_integrity_gate` | Language/Release | Normative spec suite and blocking spec integrity gate are in place; runtime implementation parity for deferred surfaces tracked separately |
| Determinism | LOCAL-PASS | workflow `v1-release-gates.yml` job `determinism_and_core_smokes` | Compiler/CI | `phase2_native` + `frontend_fixtures` deterministic suites pass locally |
| Contract runtime enforcement | LOCAL-PASS | workflow `v1-release-gates.yml` job `contract_runtime_enforcement_gate`, tests `phase2_native::{run_enforces_native_require_contracts, produced_binary_enforces_native_ensure_contracts}`, `contract_runtime_enforcement` | Compiler/Runtime | Native `@require/@ensure` checks now execute in `vibe run` and produced binaries with deterministic failure markers |
| Compiler self-host readiness | LOCAL-PASS | `reports/v1/selfhost_readiness.md`, `reports/v1/selfhost_m2_readiness.md`, `reports/v1/selfhost_m3_expansion.md`, workflow `v1-release-gates.yml` jobs `selfhost_readiness_gate`, `selfhost_m2_gate`, `selfhost_m3_shadow_gate`, `selfhost_m4_rc_cycle_gate`, `selfhost_transition_gate` | Compiler/Release | M1/M2/M3 parity and M4 promoted RC candidate + fallback drill are wired with blocking transition gate evidence |
| Native dynamic containers (`Str`/`List`/`Map`) | LOCAL-PASS | `reports/v1/dynamic_containers_conformance.md`, workflow `v1-release-gates.yml` job `dynamic_containers_gate`, `docs/development_checklist.md` section `7.3.f.1` | Compiler/Runtime | `7.3.f.1` closeout surface implemented for v1 freeze scope with deterministic parser/type/runtime evidence |
| Phase 11.1 containers/text expansion | LOCAL-PASS | `reports/v1/phase11_containers_text_readiness.md`, workflow `v1-release-gates.yml` job `phase11_containers_text_gate`, artifact `v1-phase11-containers-text` | Compiler/Runtime | Native deterministic `for in`, UTF-8-safe string index/slice checks, and container/string equality conformance are now release-gated |
| Phase 11.2 async/thread expansion | LOCAL-PASS | `reports/v1/phase11_async_thread_readiness.md`, workflow `v1-release-gates.yml` job `phase11_async_thread_gate`, artifact `v1-phase11-async-thread` | Compiler/Runtime | Async/await/thread syntax+IR flow, thread sendability guardrails, timeout/closed-channel behavior, and failure propagation checks are now release-gated |
| Phase 11.3 module/program composition | LOCAL-PASS | `reports/v1/phase11_module_composition_readiness.md`, workflow `v1-release-gates.yml` job `phase11_module_composition_gate`, artifact `v1-phase11-module-composition`, docs `docs/module/composition_guide.md`, `docs/module/migration_and_compatibility.md` | Compiler/CLI/Docs | Deterministic module/import/package-boundary diagnostics, cycle/visibility checks, and service/CLI/library template scaffolds are now release-gated |
| Ownership/sendability safety | LOCAL-PASS | `crates/vibe_cli/tests/phase7_v1_tightening.rs` + ownership fixtures | Compiler | Unknown sendability in `go` now fails closed (`E3201`) |
| Coverage thresholds | LOCAL-PASS | `docs/testing/coverage_policy.md`, `tooling/metrics/collect_phase6_metrics.py`, `tooling/metrics/validate_v1_quality_budgets.py`, workflow `v1-release-gates.yml` jobs `metrics_threshold_smoke`, `quality_and_coverage_gate` | QA/CI | Clean/no-op/incremental compile and memory-lane thresholds are validated with explicit numeric budgets |
| Rebuild reproducibility + toolchain pinning | LOCAL-PASS | workflow `v1-release-gates.yml` job `bit_identical_rebuild_gate`, workflow `v1-packaged-release.yml`, `rust-toolchain.toml` | Compiler/Release/CI | Release workflows are pinned to `1.85.1`; clean rebuild hash parity and toolchain evidence artifacts are emitted |
| Soak stability | LOCAL-PASS | workflow `v1-release-gates.yml` jobs `runtime_and_concurrency_smokes`, `memory_gc_default_gate` | Runtime | Bounded runtime/concurrency and default GC/valgrind memory lanes pass in local verification |
| Packaging integrity | LOCAL-PASS | workflow `v1-release-gates.yml` job `packaging_integrity_smoke` | Release/Tooling | Checksum generation gate added |
| Independent no-Cargo install | VALIDATED | `reports/v1/install_independence.md`, `reports/v1/phase8_ci_evidence.md`, workflow `v1-release-gates.yml` jobs `independent_install_gate`, `linux_compatibility_gate`, workflow `v1-packaged-release.yml` jobs `install_smoke_linux`, `install_smoke_linux_latest`, `install_smoke_macos`, `install_smoke_windows` | Release/CI | Hosted cross-platform packaged install cycle and Linux compatibility lanes are validated for Phase 8 closure |
| Distribution trust stack (signature/provenance/SBOM) | VALIDATED | `reports/v1/distribution_readiness.md`, `reports/v1/phase8_ci_evidence.md`, workflow `v1-packaged-release.yml` jobs `packaged_reproducibility`, `sign_attest_and_sbom` | Release/Security/CI | Hosted signed/provenance/SBOM artifact cycle is now validated for Phase 8 closure |
| CLI help/version UX | VALIDATED | `docs/cli/help_manual.md`, `docs/cli/version_output.md`, tests `cli_help_snapshots`, `cli_version`, workflow `v1-cli-ux.yml`, `reports/v1/phase8_ci_evidence.md` | CLI/Docs | Regression-tested help/version UX is validated in hosted CI |
| Compatibility (upgrade/downgrade) | LOCAL-PASS | workflow `v1-release-gates.yml` job `compatibility_gate` | CLI/Release | Adjacent compatibility path currently represented by extension/lock-mode compatibility tests |
| Ops docs readiness | DONE | docs under `docs/release/`, `docs/support/`, `docs/privacy/` | Release | Required docs added |

## Open Exceptions

| ID | Severity | Owner | Mitigation | Due Date | Status |
| --- | --- | --- | --- | --- | --- |
| V1-P0-CRUNTIME | P0 | Compiler/Runtime | Native runtime-path contract enforcement landed with blocking gate `contract_runtime_enforcement_gate` and summary artifact `v1-contract-runtime-enforcement` | 2026-02-28 | Closed |
| V1-P0-INSTALL-INDEPENDENCE | P0 | Release/CI | Complete first successful cross-platform `v1-packaged-release.yml` install smoke cycle and Linux compatibility lanes with attached run evidence | 2026-03-05 | Closed |
| V1-P0-DISTRIBUTION-TRUST | P0 | Release/Security/CI | Complete first successful signed/provenance/SBOM artifact cycle and attach run evidence | 2026-03-05 | Closed |
| V1-P1-MEMTOOLS | P1 | Runtime/CI | Default release cycle now executes GC + valgrind lanes in `memory_gc_default_gate` with artifact `v1-memory-gc-default-gate` | 2026-02-28 | Closed |
| V1-P1-RCWORKFLOW | P1 | Release | Attach first successful `v1-release-gates.yml` run URL and artifacts to checklist/dashboard | 2026-02-24 | Closed |

## Required Report Links

- `reports/v1/release_candidate_checklist.md`
- `reports/v1/readiness_dashboard.md`
- `reports/v1/smoke_validation.md`
- `reports/v1/selfhost_readiness.md`
- `reports/v1/selfhost_m2_readiness.md`
- `reports/v1/selfhost_m3_expansion.md`
- `reports/v1/spec_readiness.md`
- `reports/v1/dynamic_containers_conformance.md`
- `reports/v1/phase11_containers_text_readiness.md`
- `reports/v1/phase11_async_thread_readiness.md`
- `reports/v1/phase11_module_composition_readiness.md`
- `reports/v1/install_independence.md`
- `reports/v1/distribution_readiness.md`
- `reports/v1/phase8_ci_evidence.md`
- `reports/v1/phase8_closeout_summary.md`
- Additional gate artifacts produced by `.github/workflows/v1-release-gates.yml`
