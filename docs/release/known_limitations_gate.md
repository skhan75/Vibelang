# Known Limitations Publication Gate

Date: 2026-02-20

## Purpose

Ensure each release candidate publishes an up-to-date limitations view before promotion.

## Required Inputs

- `docs/targets/limitations_register.md`
- `docs/targets/support_matrix.md`
- candidate readiness state in `reports/v1/readiness_dashboard.md`

## Gate Rules

- Limitations register must be reviewed for each RC cycle.
- New high-severity limitations must be explicitly listed in RC checklist.
- Closed limitations require linked evidence and closure date.

## CI Enforcement

`v1-release-gates.yml` should fail if:

- limitations register is missing,
- required review marker/date is missing,
- readiness dashboard lacks limitations section updates for the candidate.

## Publication Requirement

Release notes must include:

- current high-severity limitations,
- important target/runtime caveats,
- workaround or mitigation notes where applicable.
