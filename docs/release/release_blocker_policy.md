# Release Blocker Policy

Date: 2026-02-20

## Purpose

Define which issues block v1 release and how merge/release gates enforce them.

## Severity Definitions

### P0 (Release Blocker)

Any issue that can break correctness, determinism, safety, or release integrity:

- Non-deterministic compile output for same source/toolchain
- Compile correctness regressions in core language paths
- Safety regressions in concurrency/ownership checks
- Missing/failed release integrity checks (checksums/signatures/provenance, where required)
- Critical CI gate failure in `v1-release-gates.yml`

### P1 (Must Fix Before Final GA Unless Exception)

High-impact issue with bounded workaround but material release risk:

- Major performance regression beyond threshold
- Missing non-critical but required docs/process artifacts for operational readiness
- Flaky high-signal test suites affecting confidence in rc promotion

## Merge Gate Alignment

- PRs to release branches must not introduce new open `P0`.
- PRs with open `P1` require explicit approver note and follow-up issue.
- Release PR must include links to all v1 gate reports.

## Exception Process

If a `P1` is temporarily waived:

1. Create tracked exception entry in `reports/v1/readiness_dashboard.md`
2. Add owner + due date + mitigation
3. Require release owner approval

`P0` waivers are not allowed for v1 GA.
