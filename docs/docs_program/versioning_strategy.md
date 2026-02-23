# Documentation Versioning Strategy

Date: 2026-02-22

## Branches And Labels

- `latest`:
  - current development branch documentation.
- `v1.x`:
  - release-aligned docs for supported v1 line.
- `archived`:
  - historical snapshots retained for traceability.

## Compatibility Policy

- Book and reference docs for `v1.x` must match release behavior.
- Breaking changes require:
  1. migration guide update under `docs/migrations/`,
  2. explicit release note entry,
  3. updated docs snippets and quality report.

## Release Cadence Integration

- Every RC/GA cycle publishes:
  - `reports/docs/documentation_quality.md`,
  - docs coverage snapshot,
  - snippet validation status.
