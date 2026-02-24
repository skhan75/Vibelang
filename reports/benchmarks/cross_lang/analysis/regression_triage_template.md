# Cross-Language Regression Triage Template

Use this template whenever a benchmark budget gate fails or a rerun warning is triggered.

## Incident Header

- incident_id:
- detected_at_utc:
- detected_by:
- profile: `quick` / `full`
- baseline_results:
- candidate_results:
- delta_report:

## Failing Signals

- budget_violations:
- rerun_warnings:
- impacted_cases:
- impacted_geomean_ratios:

## Reproduction Checklist

- [ ] rerun the same profile on the same machine
- [ ] rerun the same profile on native Linux lane
- [ ] confirm parity checksums/ops still match
- [ ] inspect trend/delta artifacts for persistent vs noisy movement
- [ ] check CI/environment metadata for drift (CPU governor, cores, memory, revisions)

## Root Cause Analysis

- suspected_component: runtime map / runtime channel / compiler lowering / codegen / other
- first_bad_change:
- evidence_links:
- confidence: low / medium / high

## Resolution Plan

- owner:
- mitigation_path: optimize / revert / guard / follow-up patch
- rollback_required: yes / no
- acceptance_check:

## Closure

- final_status: resolved / accepted-risk / rolled-back
- verification_links:
- follow_up_items:
