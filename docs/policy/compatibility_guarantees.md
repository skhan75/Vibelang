# Consolidated Compatibility Guarantees (v1.x)

Date: 2026-02-22

## Scope

This document consolidates compatibility guarantees across language, CLI,
runtime packaging, and security operations.

## Language and Source Compatibility

- Stable language constructs remain source-compatible across v1.x.
- Breaking behavior changes require migration guide entries under
  `docs/migrations/` and release-note visibility.

## CLI and Tooling Compatibility

- User-visible CLI changes require release-note and help-doc updates.
- Lockfile schema changes require migration path and compatibility tests.

## Runtime and Distribution Compatibility

- Tier-1 target support and compatibility expectations are governed by
  `docs/targets/support_matrix.md`.
- Linux runtime ABI expectations are governed by
  `docs/release/linux_runtime_compatibility_policy.md`.

## Security and Emergency Exception Path

- Emergency security response is governed by:
  - `docs/security/cve_response_workflow.md`
  - `docs/security/disclosure_policy.md`
- Security-driven compatibility exceptions must include:
  - explicit rationale,
  - mitigation and migration guidance,
  - deterministic regression evidence.

## Release Enforcement

- RC/GA promotion requires:
  - compatibility gate pass in release workflow,
  - known limitations review,
  - release-note sections for breaking changes and migration notes.
