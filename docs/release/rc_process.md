# Release Candidate Process

Date: 2026-02-20

## Purpose

Define how VibeLang progresses from `rc1` to GA with objective promote/reject criteria.

## RC Sequence

### RC1

- Objective: validate all v1 release gates with first candidate artifact set.
- Requirements:
  - `v1-release-gates.yml` passes with no open `P0`.
  - Release checklist first run completed.
  - Known limitations reviewed and published.

### RC2

- Objective: validate fixes from RC1 and prove stability over another full cycle.
- Requirements:
  - All RC1 blockers closed.
  - Repeat full gate pass.
  - No new `P0`; `P1` only with approved exceptions.

## Promote / Reject Criteria

Promote RC when all are true:

- All required gate jobs pass.
- No open `P0` blockers.
- All required reports linked and complete.
- Rollback plan reviewed for current candidate.

Reject RC when any are true:

- Any gate marked blocking fails.
- New `P0` introduced during candidate window.
- Required evidence artifacts are missing or stale.

## Candidate Window

- Minimum soak window: 24h for each RC cycle.
- During soak window, all regressions are triaged under `P0/P1` policy.

## Required Artifacts Per RC

- `reports/v1/release_candidate_checklist.md`
- `reports/v1/readiness_dashboard.md`
- CI artifacts from `.github/workflows/v1-release-gates.yml`
