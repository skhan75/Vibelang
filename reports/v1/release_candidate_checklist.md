# V1 Release Candidate Checklist

Candidate: `v1.0.0-rc1-dryrun-local`  
Date: 2026-02-20  
Owner: `vibelang-core`

## Gate Summary

- [ ] `v1-release-gates.yml` passed for current candidate
- [ ] No open `P0` blockers
- [x] `P1` exceptions documented and approved

## Determinism and Correctness

- [x] Deterministic build smoke passed
- [x] Frontend determinism smoke passed
- [ ] Native contract enforcement smoke passed
- [x] Compiler self-host readiness gate passed (`7.3.e`: M1 parity + one RC dry-run evidence cycle)
- [ ] Native dynamic container smoke passed (`Str`/`List`/`Map` construction + member/container lowering without `E3401`/`E3402`) after self-host readiness gate
- [x] Intent verifier-gate smoke passed

## Runtime and Stability

- [x] Concurrency deterministic/bounded smoke passed
- [x] Runtime bounded soak tests passed
- [ ] Memory/leak smoke checks passed (if enabled in this cycle)
- [ ] GC-specific smoke checks passed (only when GC runtime path is active)

## Packaging and Compatibility

- [x] Release binary checksum generated and verified
- [x] Compatibility tests (upgrade/downgrade) passed (when enabled)

## Operational Readiness

- [x] RC process reviewed (`docs/release/rc_process.md`)
- [x] Rollback plan reviewed (`docs/release/rollback_playbook.md`)
- [x] Known limitations gate reviewed (`docs/release/known_limitations_gate.md`)
- [x] Telemetry/privacy statement reviewed (`docs/privacy/telemetry_statement.md`)

## Evidence Links

- [x] `reports/v1/readiness_dashboard.md`
- [x] `reports/v1/selfhost_readiness.md`
- [ ] Workflow run URL: `TBD`
- [x] Additional artifact links: `reports/v1/smoke_validation.md`, `reports/v1/selfhost_readiness.json`

## Promote / Reject Decision

- Decision: `not-ready-for-ga`
- Decision owner: `vibelang-core`
- Notes: `P0 native contract enforcement and P0 native dynamic container support (`Str`/`List`/`Map`) remain open; memory/GC tool lanes are feature-gated and not executed in default local dry-run.`
