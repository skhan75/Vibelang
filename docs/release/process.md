# Release Process (Dry-Run First)

Date: 2026-02-17

## Goals

- Keep releases deterministic and auditable.
- Ensure policy/docs/migrations are updated before publishing.

## Release Steps

1. Run full CI and ensure all phase workflows are green.
2. Run consolidated release gates workflow (`.github/workflows/v1-release-gates.yml`).
3. Execute the current RC process in `docs/release/rc_process.md`.
4. Update `CHANGELOG.md` with release notes and migration entries.
5. Verify migration guide examples compile/run.
6. Review limitations gate (`docs/release/known_limitations_gate.md`).
7. Confirm rollback readiness (`docs/release/rollback_playbook.md`).
8. Publish tag and release artifacts after approvals.

## Changelog Requirements

Each release section should include:

- Added
- Changed
- Fixed
- Migration Notes

## Required Evidence Bundle

Every release candidate should include links to:

- `.yb` parity report (`reports/phase6/source_extension_migration.md`)
- self-host conformance report (`reports/phase6/self_hosting_conformance.md`)
- metrics artifacts under `reports/phase6/metrics/`
- support matrix and known limitations docs
- v1 readiness dashboard (`reports/v1/readiness_dashboard.md`)
- v1 release candidate checklist (`reports/v1/release_candidate_checklist.md`)
- self-host readiness report (`reports/v1/selfhost_readiness.md`)
- spec readiness report (`reports/v1/spec_readiness.md`)

## Required Operational Docs

- `docs/release/rc_process.md`
- `docs/release/rollback_playbook.md`
- `docs/release/release_blocker_policy.md`
- `docs/release/known_limitations_gate.md`
- `docs/support/issue_triage_sla.md`
- `docs/privacy/telemetry_statement.md`
