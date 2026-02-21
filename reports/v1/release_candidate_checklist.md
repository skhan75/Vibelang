# V1 Release Candidate Checklist

Candidate: `v1.0.0-rc1-dryrun-local`  
Date: 2026-02-21  
Owner: `vibelang-core`

## Gate Summary

- [ ] `v1-release-gates.yml` passed for current candidate
- [ ] No open `P0` blockers
- [x] `P1` exceptions documented and approved

## Determinism and Correctness

- [x] Deterministic build smoke passed
- [x] Frontend determinism smoke passed
- [x] Spec integrity gate baseline passed locally (`spec_integrity_gate`: markdown lint + consistency + coverage validators)
- [ ] Native contract enforcement smoke passed
- [x] Compiler self-host readiness gate passed (`7.3.e`: M1 parity + one RC dry-run evidence cycle)
- [x] Native dynamic container smoke passed (`Str`/`List`/`Map` construction + member/container lowering without `E3401`/`E3402`) after self-host readiness gate
- [x] Intent verifier-gate smoke passed

## Runtime and Stability

- [x] Concurrency deterministic/bounded smoke passed
- [x] Runtime bounded soak tests passed
- [ ] Memory/leak smoke checks passed (if enabled in this cycle)
- [ ] GC-specific smoke checks passed (only when GC runtime path is active)

## Packaging and Compatibility

- [x] Release binary checksum generated and verified
- [ ] Tier-1 packaged install smokes passed on clean machines (Linux/macOS/Windows; no Cargo dependency)
- [ ] Signed artifact trust bundle passed (signature + provenance + SBOM validation)
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
- [x] `reports/v1/install_independence.md`
- [x] `reports/v1/distribution_readiness.md`
- [ ] Workflow run URL: `TBD`
- [x] Additional artifact links: `reports/v1/smoke_validation.md`, `reports/v1/dynamic_containers_conformance.md`, `reports/v1/install_independence.md`, `reports/v1/distribution_readiness.md`, `reports/v1/selfhost_readiness.json`, `reports/v1/spec_readiness.md`

## Promote / Reject Decision

- Decision: `not-ready-for-ga`
- Decision owner: `vibelang-core`
- Notes: `P0 native contract enforcement remains open; independent packaged install and signed distribution trust stack await first successful cross-platform CI evidence cycle; memory/GC tool lanes are feature-gated and not executed in default local dry-run.`
