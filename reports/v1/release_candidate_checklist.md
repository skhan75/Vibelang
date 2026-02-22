# V1 Release Candidate Checklist

Candidate: `v1.0.0-rc1-dryrun-local`  
Date: 2026-02-21  
Owner: `vibelang-core`

## Gate Summary

- [ ] `v1-release-gates.yml` passed for current candidate
- [x] No open `P0` blockers
- [x] `P1` exceptions documented and approved

## Determinism and Correctness

- [x] Deterministic build smoke passed
- [x] Frontend determinism smoke passed
- [x] Spec integrity gate baseline passed locally (`spec_integrity_gate`: markdown lint + consistency + coverage validators)
- [x] Native contract enforcement smoke passed (`contract_runtime_enforcement_gate`, artifact `v1-contract-runtime-enforcement`)
- [x] Compiler self-host readiness gate passed (`7.3.e`: M1 parity + one RC dry-run evidence cycle)
- [x] M4 promoted self-host RC cycle passed for diagnostics ordering candidate with rollback drill (`selfhost_m4_rc_cycle_gate`, artifact `v1-selfhost-m4-rc-cycle`)
- [x] Native dynamic container smoke passed (`Str`/`List`/`Map` construction + member/container lowering without `E3401`/`E3402`) after self-host readiness gate
- [x] Phase 11.1 containers/text conformance passed (`phase11_containers_text_gate`, artifact `v1-phase11-containers-text`)
- [x] Phase 11.2 async/thread conformance passed (`phase11_async_thread_gate`, artifact `v1-phase11-async-thread`)
- [x] Phase 11.3 module/program composition conformance passed (`phase11_module_composition_gate`, artifact `v1-phase11-module-composition`)
- [x] Phase 12.1 stdlib conformance passed locally (`phase12_stdlib`, `reports/v1/phase12_stdlib_readiness.md`)
- [x] Phase 12.2 package lifecycle conformance passed locally (`phase12_package_ecosystem`, `vibe_pkg` unit suite, `reports/v1/phase12_package_ecosystem_readiness.md`)
- [x] Phase 12.3 QA ecosystem conformance passed locally (`phase12_test_ergonomics`, coverage/golden tooling checks, `reports/v1/phase12_qa_ecosystem_readiness.md`)
- [x] Intent verifier-gate smoke passed

## Runtime and Stability

- [x] Concurrency deterministic/bounded smoke passed
- [x] Runtime bounded soak tests passed
- [x] Memory/leak smoke checks passed (`memory_gc_default_gate`, valgrind lane)
- [x] GC-specific smoke checks passed (`memory_gc_default_gate`, GC-observable lane)

## Packaging and Compatibility

- [x] Release binary checksum generated and verified
- [x] Tier-1 packaged install smokes passed on clean machines (Linux/macOS/Windows; no Cargo dependency)
- [x] Signed artifact trust bundle passed (signature + provenance + SBOM validation)
- [x] Compatibility tests (upgrade/downgrade) passed (when enabled)

## Operational Readiness

- [x] RC process reviewed (`docs/release/rc_process.md`)
- [x] Rollback plan reviewed (`docs/release/rollback_playbook.md`)
- [x] Known limitations gate reviewed (`docs/release/known_limitations_gate.md`)
- [x] Telemetry/privacy statement reviewed (`docs/privacy/telemetry_statement.md`)

## Evidence Links

- [x] `reports/v1/readiness_dashboard.md`
- [x] `reports/v1/selfhost_readiness.md`
- [x] `reports/v1/spec_readiness.md`
- [x] `reports/v1/phase11_containers_text_readiness.md`
- [x] `reports/v1/phase11_async_thread_readiness.md`
- [x] `reports/v1/phase11_module_composition_readiness.md`
- [x] `reports/v1/phase12_stdlib_readiness.md`
- [x] `reports/v1/phase12_package_ecosystem_readiness.md`
- [x] `reports/v1/phase12_qa_ecosystem_readiness.md`
- [x] `reports/v1/install_independence.md`
- [x] `reports/v1/distribution_readiness.md`
- [x] `reports/v1/phase8_ci_evidence.md`
- [x] `docs/selfhost/m4_transition_criteria.md`
- [x] `docs/release/selfhost_transition_playbook.md`
- [x] Workflow run URL: hosted CI evidence captured for current candidate cycle (`v1-packaged-release.yml`, `v1-cli-ux.yml`)
- [x] Additional artifact links: `reports/v1/smoke_validation.md`, `reports/v1/dynamic_containers_conformance.md`, `reports/v1/phase11_containers_text_readiness.md`, `reports/v1/phase11_async_thread_readiness.md`, `reports/v1/phase11_module_composition_readiness.md`, `reports/v1/phase12_stdlib_readiness.md`, `reports/v1/phase12_package_ecosystem_readiness.md`, `reports/v1/phase12_qa_ecosystem_readiness.md`, `reports/phase12/coverage_summary.json`, `reports/phase12/health_status.md`, `reports/v1/install_independence.md`, `reports/v1/distribution_readiness.md`, `reports/v1/phase8_ci_evidence.md`, `reports/v1/phase8_closeout_summary.md`, `reports/v1/selfhost_readiness.json`, `reports/v1/spec_readiness.md`, artifacts `v1-selfhost-m4-rc-cycle`, `v1-phase11-containers-text`, `v1-phase11-async-thread`, `v1-phase11-module-composition`

## Promote / Reject Decision

- Decision: `not-ready-for-ga`
- Decision owner: `vibelang-core`
- Notes: `Phase 10 hardening items are locally validated; final GA promotion still requires hosted candidate run with all gates green and linked workflow evidence.`
