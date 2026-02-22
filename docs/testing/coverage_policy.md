# Test Coverage Policy (v1)

Date: 2026-02-20

## Purpose

Define minimum coverage expectations for v1 release gating.

## Required Test Pillars

- Frontend/diagnostics goldens (`frontend_fixtures.rs`)
- Single-thread and concurrency behavior samples (`phase7_validation.rs`, `phase7_concurrency.rs`)
- Intent linting and verifier gate (`phase7_intent_validation.rs`)
- v1 tightening smokes (algorithmic recursion, memory pressure, ownership safety)
  (`phase7_v1_tightening.rs`)
- Phase 12 stdlib surface suite (`phase12_stdlib.rs`)
- Phase 12 package lifecycle suite (`phase12_package_ecosystem.rs`)
- Phase 12 test ergonomics suite (`phase12_test_ergonomics.rs`)

## Minimum Gate Expectations

- No regressions in deterministic output tests.
- Bounded stress tests remain within documented runtime budget.
- Ownership/sendability negative cases must continue producing expected diagnostics.
- Feature-gated memory/GC instrumentation tests are optional by default and required in
  specialized memory-tooling lanes.

## Required Validators

- `python3 tooling/metrics/validate_phase7_coverage_matrix.py`
- `python3 tooling/metrics/validate_v1_quality_budgets.py`
- `python3 tooling/coverage/collect_phase12_coverage.py`
- `python3 tooling/coverage/validate_phase12_coverage.py`

## Policy Updates

Coverage policy changes require:

1. Update in this document.
2. Corresponding workflow gate updates.
3. Readiness dashboard note for release cycle impact.
