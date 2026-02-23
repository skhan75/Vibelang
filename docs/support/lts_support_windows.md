# VibeLang LTS and Support Windows (v1.x)

Date: 2026-02-22

## Objectives

- Provide predictable maintenance expectations for production teams.
- Align support commitments with release and security governance.

## Support Window Model

- **Current Stable (v1.x latest)**
  - Feature and bug-fix updates.
  - Security patches.
- **LTS Track (designated v1.x releases)**
  - Security and critical bug fixes only.
  - No feature backports except approved critical exceptions.

## Durations

- LTS maintenance window: **24 months** from LTS release date.
- Security-only extended window: **12 months** after LTS maintenance window.
- End-of-life notice lead time: **>= 90 days** before EOL date.

## Compatibility Guarantees

- Source compatibility is preserved across v1.x stable/LTS except documented
  security exceptions.
- Lockfile and CLI behavior changes require migration notes and release note
  entries.
- Breaking changes outside major version require explicit exception signoff.

## Escalation and Exceptions

- Emergency security fixes may prioritize safety over strict behavior parity.
- Any compatibility-impacting emergency patch must include:
  - disclosure rationale,
  - migration guidance,
  - deterministic regression tests.

## Evidence

- `docs/policy/compatibility_guarantees.md`
- `reports/v1/lts_support_exercise.md`
