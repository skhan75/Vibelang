# V1 Readiness Dashboard

Date: 2026-03-25

## Overall Status

- Release candidate cycle: `v1.1.1-stdlib-migration`
- Blocking `P0` gates open: `0`
- `P1` gates open: `0`

## v1.1.1 Patch Release — Stdlib C-to-YB Migration

### Summary

Migrated ~100 hardcoded C runtime stdlib functions from compiler-wired tables into
self-hosted `.yb` modules under `stdlib/std/`, using `@native` FFI for C-backed
primitives. This is a compiler architecture improvement with no user-facing API changes.

### Modules Migrated

| Module | Functions | Implementation |
|--------|-----------|----------------|
| `std.path` | 4 | Pure VibeLang |
| `std.log` | 3 | `@native` wrappers |
| `std.env` | 3 | `@native` wrappers |
| `std.cli` | 2 | `@native` wrappers |
| `std.time` | 4 | `@native` wrappers |
| `std.fs` | 4 | `@native` wrappers |
| `std.convert` | 10 | `@native` wrappers |
| `std.text` | 10 | `@native` wrappers |
| `std.encoding` | 6 | `@native` wrappers |
| `std.regex` | 2 | `@native` wrappers |
| `std.net` | 8 | `@native` wrappers |
| `std.math` | 1 | `@native` wrapper |
| `std.str_builder` | 4 | `@native` wrappers |
| `std.json` | 12 | `@native` wrappers |
| `std.http` | 10 + 2 types | `@native` wrappers |

### Compiler Special Cases Retained

- `json.encode`/`json.decode`: compile-time struct schema generation
- `json.builder.*`: special `JsonBuilder` type
- `json.from_map`: special map-to-JSON conversion
- `simd.*`: Cranelift SIMD intrinsics
- `bench.*`: benchmark feature-gated functions

### Test Evidence

- All 22 stdlib example programs pass (`examples/07_stdlib_io_json_regex_http/` + module import test)
- `cargo test --lib` passes
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- CI checks: `fmt_lint`, `cargo-deny`, `secret-scan` all pass

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
| Phase 12.1 stdlib ecosystem surface | LOCAL-PASS | `reports/v1/phase12_stdlib_readiness.md`, tests `phase12_stdlib`, fixtures `compiler/tests/fixtures/stdlib/*` | Compiler/Runtime/Docs | Time/path/fs/json/http essentials are wired through checker/codegen/runtime with deterministic/error-model tests and module docs |
| Phase 12.2 package lifecycle and registry | LOCAL-PASS | `reports/v1/phase12_package_ecosystem_readiness.md`, tests `phase12_package_ecosystem`, `vibe_pkg` unit suite | CLI/Package/Security | Publish/index flow, audit policy enforcement, and semver upgrade tooling are implemented with local conformance evidence |
| Phase 12.3 QA ecosystem expansion | LOCAL-PASS | `reports/v1/phase12_qa_ecosystem_readiness.md`, `reports/phase12/coverage_summary.json`, tests `phase12_test_ergonomics` | QA/CLI | `vibe test` filter/shard/report ergonomics, coverage collectors/validators, and golden update tooling are in place |
| Ownership/sendability safety | LOCAL-PASS | `crates/vibe_cli/tests/phase7_v1_tightening.rs` + ownership fixtures | Compiler | Unknown sendability in `go` now fails closed (`E3201`) |
| Coverage thresholds | LOCAL-PASS | `docs/testing/coverage_policy.md`, `tooling/metrics/collect_phase6_metrics.py`, `tooling/metrics/validate_v1_quality_budgets.py`, workflow `v1-release-gates.yml` jobs `metrics_threshold_smoke`, `quality_and_coverage_gate` | QA/CI | Clean/no-op/incremental compile and memory-lane thresholds are validated with explicit numeric budgets |
| Rebuild reproducibility + toolchain pinning | LOCAL-PASS | workflow `v1-release-gates.yml` job `bit_identical_rebuild_gate`, workflow `v1-packaged-release.yml`, `rust-toolchain.toml` | Compiler/Release/CI | Release workflows are pinned to `1.85.1`; clean rebuild hash parity and toolchain evidence artifacts are emitted |
| Soak stability | LOCAL-PASS | workflow `v1-release-gates.yml` jobs `runtime_and_concurrency_smokes`, `memory_gc_default_gate` | Runtime | Bounded runtime/concurrency and default GC/valgrind memory lanes pass in local verification |
| Packaging integrity | LOCAL-PASS | workflow `v1-release-gates.yml` job `packaging_integrity_smoke` | Release/Tooling | Checksum generation gate added |
| Independent no-Cargo install | VALIDATED | `reports/v1/install_independence.md`, `reports/v1/phase8_ci_evidence.md`, workflow `v1-release-gates.yml` jobs `independent_install_gate`, `linux_compatibility_gate`, workflow `v1-packaged-release.yml` jobs `install_smoke_linux`, `install_smoke_linux_latest`, `install_smoke_macos`, `install_smoke_windows` | Release/CI | Hosted cross-platform packaged install cycle and Linux compatibility lanes are validated for Phase 8 closure |
| Distribution trust stack (signature/provenance/SBOM) | VALIDATED | `reports/v1/distribution_readiness.md`, `reports/v1/phase8_ci_evidence.md`, workflow `v1-packaged-release.yml` jobs `packaged_reproducibility`, `sign_attest_and_sbom` | Release/Security/CI | Hosted signed/provenance/SBOM artifact cycle is now validated for Phase 8 closure |
| CLI help/version UX | VALIDATED | `docs/cli/help_manual.md`, `docs/cli/version_output.md`, tests `cli_help_snapshots`, `cli_version`, workflow `v1-cli-ux.yml`, `reports/v1/phase8_ci_evidence.md` | CLI/Docs | Regression-tested help/version UX is validated in hosted CI |
| CI workflow cost controls | VALIDATED | `reports/v1/ci_cost_optimization.md`, workflow headers under `.github/workflows/*.yml` | Release/CI | Branch/path scoping, concurrency cancellation, Rust cache, PR packaging reduction, and short artifact retention are now enforced |
| Compatibility (upgrade/downgrade) | LOCAL-PASS | workflow `v1-release-gates.yml` job `compatibility_gate` | CLI/Release | Adjacent compatibility path currently represented by extension/lock-mode compatibility tests |
| Ops docs readiness | DONE | docs under `docs/release/`, `docs/support/`, `docs/privacy/` | Release | Required docs added |

## Phase 13.2/13.3/14 Gate Snapshot

| Gate | Checklist Source | Status | Owner | Planned Evidence Path |
| --- | --- | --- | --- | --- |
| VG-005 Unsafe escape hatch syntax/boundaries | Guardrails: Escape Hatches | LOCAL-PASS | Language/Compiler | `docs/spec/unsafe_escape_hatches.md`, test `phase2_native::build_emits_unsafe_audit_and_allocation_profile_artifacts` |
| VG-006 Unsafe review path required | Guardrails: Escape Hatches | LOCAL-PASS | Compiler/Release | `docs/release/unsafe_review_policy.md`, test `phase2_native::build_rejects_unsafe_blocks_without_review_reference`, job `unsafe_governance_gate` |
| VG-007 Unsafe block audit report per build | Guardrails: Escape Hatches | LOCAL-PASS | CLI/Compiler | `write_unsafe_audit_report`, `unsafe_governance_gate` artifact |
| VG-008 Allocation visibility in diagnostics/profile outputs | Guardrails: Transparent Performance Model | LOCAL-PASS | Compiler/Runtime | `reports/v1/allocation_visibility_smoke.json`, job `allocation_visibility_gate` |
| VG-009 CPU/memory/latency release benchmark publication | Guardrails: Transparent Performance Model | LOCAL-PASS | Runtime/Tooling | `reports/v1/release_benchmarks.json`, job `release_benchmark_publication_gate` |
| VG-010 Cost model docs for copies/alloc/concurrency | Guardrails: Transparent Performance Model | LOCAL-PASS | Language Docs | `docs/spec/cost_model.md` |
| VG-017 Debug/profiling workflow | `13.2.1` | LOCAL-PASS | Compiler/Runtime/DX | `docs/debugging/workflow.md`, `reports/phase13/debugging_workflow.md`, job `debug_profile_workflow_gate` |
| VG-018 Runtime observability primitives | `13.2.2` | LOCAL-PASS | Runtime/Tooling | `docs/observability/contracts.md`, `reports/phase13/observability_primitives.md`, job `observability_contracts_gate` |
| VG-019 Runtime incident triage playbook | `13.2.3` | LOCAL-PASS | Runtime/Release | `docs/support/production_incident_triage.md`, `reports/phase13/crash_repro_sample.md` |
| VG-020 Deterministic crash repro format | `13.2.4` | LOCAL-PASS | Compiler/Runtime/Tooling | `docs/support/crash_repro_format.md`, `reports/phase13/crash_repro_sample.json`, job `crash_repro_artifact_gate` |
| VG-021 LTS/support windows + compatibility guarantees | `13.3.1` | LOCAL-PASS | Release/Docs | `docs/support/lts_support_windows.md`, `docs/policy/compatibility_guarantees.md`, `reports/v1/lts_support_exercise.md` |
| VG-022 CVE workflow + disclosure policy | `13.3.2` | LOCAL-PASS | Security/Release | `docs/security/cve_response_workflow.md`, `docs/security/disclosure_policy.md`, `reports/v1/security_response_exercise.md` |
| VG-023 Release-notes automation | `13.3.3` | LOCAL-PASS | Release/Tooling | workflow `release-notes-automation.yml`, `reports/v1/release_notes_preview.md`, job `release_notes_automation_gate` |
| VG-024 Phase 7.4 docs/book closure | `13.3.4`, `7.4.*` | LOCAL-PASS | Docs/DX/CI | `book/`, workflow `docs-quality.yml`, `reports/docs/documentation_quality.md` |
| VG-025 Pilot application evidence | `14.1.*` | LOCAL-PASS | Product/Runtime/DX | `pilot-apps/`, `reports/phase14/pilot_metrics.json`, pilot case studies |
| VG-026 GA promotion gate evidence | `14.2.*`, Phase 13/14 exits | VALIDATED | Release/Security/CI | `https://github.com/skhan75/VibeLang/actions/runs/22302057210`, `https://github.com/skhan75/VibeLang/actions/runs/22299615440`, `reports/v1/hosted_rc_cycles.md`, `reports/v1/phase10_13_exit_audit.md`, `reports/v1/ga_freeze_bundle_manifest.md`, `reports/v1/ga_readiness_announcement.md` |

## Blocker Register (Program Work)

| ID | Severity | Owner | Risk | Due Date | Current Mitigation | Status |
| --- | --- | --- | --- | --- | --- | --- |
| V1-P1-DEBUG-OBS | P1 | Compiler/Runtime/DX | Debug/perf triage contracts and evidence collectors are now in place | 2026-03-07 | Closed via VG-017..VG-020 (`docs/debugging/workflow.md`, `reports/phase13/*`) | Closed |
| V1-P0-SEC-GOV | P0 | Security/Release | CVE/disclosure governance landed with exercise evidence and release gating | 2026-03-07 | Closed via VG-022 (`docs/security/*`, `reports/v1/security_response_exercise.md`) | Closed |
| V1-P1-RELNOTES-AUTO | P1 | Release/Tooling | Release-note automation is implemented and section-completeness enforced in CI | 2026-03-09 | Closed via VG-023 (`tooling/release/*`, `reports/v1/release_notes_preview.md`) | Closed |
| V1-P1-DOCS-74 | P1 | Docs/DX/CI | 7.4 docs/book quality gate is now automated and published | 2026-03-12 | Closed via VG-024 (`book/`, `.github/workflows/docs-quality.yml`, `reports/docs/documentation_quality.md`) | Closed |
| V1-P0-PILOT-EVIDENCE | P0 | Product/Runtime/DX | Pilot application package, metrics, and migration backlog are published | 2026-03-16 | Closed via VG-025 (`pilot-apps/`, `reports/phase14/*`) | Closed |
| V1-P0-GA-PROMOTION | P0 | Release/Security/CI | GA promotion package is complete with RC-cycle ledger, phase-exit audit, and freeze manifest | 2026-03-20 | Closed via VG-026 (`reports/v1/hosted_rc_cycles.md`, `reports/v1/ga_readiness_announcement.md`) | Closed |
| V1-P0-PUBLIC-EVIDENCE | P0 | Release/Security/CI | Hosted RC run URLs are now attached and GA evidence regeneration passes | 2026-02-24 | Closed via `reports/v1/hosted_rc_cycle_inputs.json`, `reports/v1/hosted_rc_cycles.md`, `reports/v1/ga_readiness_announcement.md` | Closed |
| V1-P0-RELEASE-PUBLISH | P0 | Release/Security/CI | Public release payload is now published (tag + release notes + signed artifacts/SBOM/provenance) | 2026-02-24 | Closed via `https://github.com/skhan75/VibeLang/releases/tag/v1.0.0` | Closed |

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
- `reports/v1/phase12_stdlib_readiness.md`
- `reports/v1/phase12_package_ecosystem_readiness.md`
- `reports/v1/phase12_qa_ecosystem_readiness.md`
- `reports/phase12/coverage_summary.json`
- `reports/phase12/health_status.md`
- `reports/v1/install_independence.md`
- `reports/v1/distribution_readiness.md`
- `reports/v1/phase8_ci_evidence.md`
- `reports/v1/phase8_closeout_summary.md`
- `reports/v1/ci_cost_optimization.md`
- `reports/v1/ga_go_no_go_checklist.md`
- Additional gate artifacts produced by `.github/workflows/v1-release-gates.yml`
