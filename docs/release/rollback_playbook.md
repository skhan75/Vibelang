# Rollback Playbook

Date: 2026-02-20

## Purpose

Provide a deterministic procedure for detecting a bad release and safely rolling back.

## Trigger Signals

Initiate rollback evaluation if any occur:

- Critical runtime regression in release smoke workloads
- Determinism regression for stable inputs
- Safety regression (`P0`) in compiler/runtime behavior
- Packaging/integrity verification failure

## Decision Matrix

- `P0` confirmed and no immediate hotfix path: rollback immediately.
- `P1` with safe mitigation and no correctness risk: may continue with exception approval.

## Rollback Steps

1. Freeze further release promotion.
2. Announce rollback start in release channel with issue reference.
3. Revert release tag/artifact pointers to last known good candidate.
4. Re-run minimum smoke suite on rollback target.
5. Publish rollback note and mitigation ETA.

## Verification After Rollback

- Core deterministic build smoke passes.
- Runtime/concurrency smoke passes.
- Intent lint and docs smoke pass.
- Readiness dashboard updated with rollback record.

## Communication Template

- Incident summary
- Impact scope
- Rollback action taken
- User-facing mitigation
- Follow-up fix timeline

## Ownership

- Release owner: executes rollback command path.
- Runtime owner: validates runtime correctness after rollback.
- Compiler owner: validates determinism/safety after rollback.
