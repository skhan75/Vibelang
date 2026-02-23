# Security Disclosure Policy

Date: 2026-02-22

## Coordinated Disclosure Window

- Standard disclosure target: **90 days** from confirmed report.
- Earlier disclosure allowed when:
  - fix is already public and exploitable details are broadly known, or
  - active exploitation risk requires immediate communication.

## Embargo Handling

- Embargo applies while patch validation is in progress.
- Access to details is limited to required maintainers/reviewers.
- All embargoed artifacts are tagged as confidential in tracking notes.

## Public Advisory Requirements

Each disclosure must include:

- affected versions and surfaces,
- severity and impact summary,
- fixed versions and upgrade path,
- temporary mitigation when immediate upgrade is unavailable.

## Release Coordination

- Security disclosure accompanies patch release notes.
- RC/GA promotion is blocked by unresolved `P0` security issues.
- Disclosure outcomes are reflected in readiness dashboard and release checklist.
