# Issue Triage SLA and Severity Taxonomy

Date: 2026-02-20

## Severity Taxonomy

- `P0` Critical
  - Breaks correctness, determinism, safety, or release integrity.
  - Requires immediate triage and active owner response.
- `P1` High
  - Major regression with workaround; blocks GA if unresolved.
- `P2` Medium
  - Significant usability/performance gap without correctness break.
- `P3` Low
  - Minor issue, docs/nits, quality improvements.

## Triage SLA Targets

- `P0`: acknowledge within 4h, owner assigned within 4h, mitigation plan within 24h
- `P1`: acknowledge within 1 business day, owner assigned within 1 business day
- `P2`: acknowledge within 3 business days
- `P3`: acknowledge within 5 business days

## Triage Flow

1. Intake and classify severity.
2. Confirm reproducibility and impact scope.
3. Assign owner and target milestone.
4. Link issue to release gate if applicable.
5. Track to closure with evidence and regression tests.

## Escalation

- Any unresolved `P0` automatically escalates to release owner.
- Repeated regressions in same area trigger mandatory postmortem note.
