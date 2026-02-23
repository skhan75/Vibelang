# CVE Response Workflow

Date: 2026-02-22

## Purpose

Define security vulnerability intake, triage, remediation, and release workflow
for VibeLang.

## Intake Channels

- Security issue submission through private maintainer contact and tracked issue.
- Advisory database entries under package security policy.

## Triage SLA

- `P0` security issues: acknowledge and owner assign within 4h.
- `P1` security issues: acknowledge within 1 business day.

## Workflow

1. **Intake**
   - capture report details and affected components.
2. **Classification**
   - severity (`P0`..`P3`) and exploitability assessment.
3. **Containment**
   - temporary mitigations and release-gate impact.
4. **Fix and Validation**
   - implement patch with deterministic regression tests.
5. **Disclosure**
   - follow `docs/security/disclosure_policy.md`.
6. **Release and Follow-Up**
   - publish advisory and release notes,
   - update known limitations and readiness dashboard where needed.

## Required Artifacts

- incident/cve exercise report (`reports/v1/security_response_exercise.md`)
- release notes entry with mitigation path
- linked regression tests
