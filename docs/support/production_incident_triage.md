# Production Incident Triage (Vibe Runtime)

Date: 2026-02-22

## Scope

Operational triage workflow for runtime failures in production Vibe services.

## Severity Mapping

Use base severity taxonomy in `docs/support/issue_triage_sla.md`.

- `P0`: correctness/safety/data-loss/runtime unavailability.
- `P1`: major regression with workaround.
- `P2/P3`: non-blocking quality/usability issues.

## Triage Steps

1. **Stabilize**
   - confirm blast radius,
   - apply rollback/mitigation if needed (`docs/release/rollback_playbook.md`).
2. **Collect Deterministic Artifacts**
   - command invocation,
   - source and binary hashes,
   - debug map + unsafe audit + alloc profile artifacts,
   - stdout/stderr and environment metadata.
3. **Classify Failure Channel**
   - `Result`-channel failure,
   - contract failure (`@require`/`@ensure`),
   - panic/trap path.
4. **Reproduce**
   - use crash repro bundle format (`docs/support/crash_repro_format.md`),
   - replay under locked mode and same toolchain where possible.
5. **Escalate**
   - route to owner based on affected surface (compiler/runtime/CLI),
   - open blocker per release policy when severity requires.

## Required Incident Artifact Bundle

- `crash_repro_sample.json`/`.md` style bundle metadata
- debug and profile artifact pointers
- mitigation and owner assignment notes

## Exercise Requirement

Each RC cycle should run at least one incident-triage drill and attach results
to release evidence.
