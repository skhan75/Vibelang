# V1 Release Candidate Checklist (Canonical)

Candidate: `<set per cycle>`  
Date: `<set per cycle>`  
Owner: `vibelang-core`

## Gate Summary

- [ ] Hosted `v1-release-gates.yml` pass URL(s) are attached for current candidate
- [ ] No open `P0` blockers
- [ ] `P1` exceptions documented and approved

## Determinism and Correctness

- [ ] Deterministic build smoke passed
- [ ] Frontend determinism smoke passed
- [ ] Spec integrity gate baseline passed (job `spec_integrity_gate`)
- [ ] Native contract enforcement smoke passed (`contract_runtime_enforcement_gate`)
- [ ] Compiler self-host readiness gate passed (`7.3.e`)
- [ ] M4 promoted self-host RC cycle passed (`selfhost_m4_rc_cycle_gate`)
- [ ] Native dynamic container smoke passed (after self-host readiness gate)
- [ ] Phase 11.1 containers/text conformance passed (`phase11_containers_text_gate`)
- [ ] Phase 11.2 async/thread conformance passed (`phase11_async_thread_gate`)
- [ ] Phase 11.3 module/program composition conformance passed (`phase11_module_composition_gate`)
- [ ] Phase 12.1 stdlib conformance passed (`phase12_stdlib`)
- [ ] Phase 12.2 package lifecycle conformance passed (`phase12_package_ecosystem`)
- [ ] Phase 12.3 QA ecosystem conformance passed (`phase12_test_ergonomics`)
- [ ] Intent verifier-gate smoke passed

## Runtime and Stability

- [ ] Concurrency deterministic/bounded smoke passed
- [ ] Runtime bounded soak tests passed
- [ ] Memory/leak smoke checks passed (`memory_gc_default_gate`)
- [ ] GC-specific smoke checks passed (`memory_gc_default_gate`)

## Packaging and Compatibility

- [ ] Release binary checksum generated and verified
- [ ] Tier-1 packaged install smokes passed (Linux/macOS/Windows; no Cargo dependency)
- [ ] Signed artifact trust bundle passed (signature + provenance + SBOM validation)
- [ ] Compatibility tests (upgrade/downgrade) passed (when enabled)

## CI Cost Efficiency (Non-Blocking)

- [ ] Workflow triggers are scoped with path filters
- [ ] Workflow-level concurrency cancellation is enabled
- [ ] Rust job caching is enabled
- [ ] PR duplication is removed
- [ ] PR packaging lane is reduced appropriately
- [ ] Artifact retention is set intentionally

## Operational Readiness

- [ ] RC process reviewed (`docs/release/rc_process.md`)
- [ ] Rollback plan reviewed (`docs/release/rollback_playbook.md`)
- [ ] Known limitations gate reviewed (`docs/release/known_limitations_gate.md`)
- [ ] Telemetry/privacy statement reviewed (`docs/privacy/telemetry_statement.md`)

## Evidence Links (attach per cycle)

- [ ] `reports/v1/readiness_dashboard.md`
- [ ] `reports/v1/ga_go_no_go_checklist.md`
- [ ] `reports/v1/selfhost_readiness.md`
- [ ] `reports/v1/spec_readiness.md`
- [ ] `reports/v1/phase11_containers_text_readiness.md`
- [ ] `reports/v1/phase11_async_thread_readiness.md`
- [ ] `reports/v1/phase11_module_composition_readiness.md`
- [ ] `reports/v1/phase12_stdlib_readiness.md`
- [ ] `reports/v1/phase12_package_ecosystem_readiness.md`
- [ ] `reports/v1/phase12_qa_ecosystem_readiness.md`
- [ ] `reports/v1/install_independence.md`
- [ ] `reports/v1/distribution_readiness.md`
- [ ] `reports/v1/phase8_ci_evidence.md`
- [ ] `reports/v1/ci_cost_optimization.md`
- [ ] Workflow run URLs for the cycle

