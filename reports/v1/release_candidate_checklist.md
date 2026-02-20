# V1 Release Candidate Checklist

Candidate: `v1.0.0-rcN`  
Date: 2026-02-20  
Owner: `TBD`

## Gate Summary

- [ ] `v1-release-gates.yml` passed for current candidate
- [ ] No open `P0` blockers
- [ ] `P1` exceptions documented and approved

## Determinism and Correctness

- [ ] Deterministic build smoke passed
- [ ] Frontend determinism smoke passed
- [ ] Native contract enforcement smoke passed
- [ ] Intent verifier-gate smoke passed

## Runtime and Stability

- [ ] Concurrency deterministic/bounded smoke passed
- [ ] Runtime bounded soak tests passed
- [ ] Memory/leak smoke checks passed (if enabled in this cycle)
- [ ] GC-specific smoke checks passed (only when GC runtime path is active)

## Packaging and Compatibility

- [ ] Release binary checksum generated and verified
- [ ] Compatibility tests (upgrade/downgrade) passed (when enabled)

## Operational Readiness

- [ ] RC process reviewed (`docs/release/rc_process.md`)
- [ ] Rollback plan reviewed (`docs/release/rollback_playbook.md`)
- [ ] Known limitations gate reviewed (`docs/release/known_limitations_gate.md`)
- [ ] Telemetry/privacy statement reviewed (`docs/privacy/telemetry_statement.md`)

## Evidence Links

- [ ] `reports/v1/readiness_dashboard.md`
- [ ] Workflow run URL: `TBD`
- [ ] Additional artifact links: `TBD`

## Promote / Reject Decision

- Decision: `TBD`
- Decision owner: `TBD`
- Notes: `TBD`
