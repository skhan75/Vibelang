# Release Process (Dry-Run First)

Date: 2026-02-17

## Goals

- Keep releases deterministic and auditable.
- Ensure policy/docs/migrations are updated before publishing.

## Release Steps

1. Run full CI and ensure all phase workflows are green.
2. Run release dry-run workflow (`.github/workflows/release.yml`).
3. Update `CHANGELOG.md` with release notes and migration entries.
4. Verify migration guide examples compile/run.
5. Publish tag and release artifacts after approvals.

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
