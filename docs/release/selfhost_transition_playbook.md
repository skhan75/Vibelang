# Selfhost Transition Playbook

Date: 2026-02-21

## Purpose

Provide the operational runbook for promoting self-host components, validating
an RC cycle, and performing immediate fallback if regressions are detected.

## Current Promotion Candidate

- Component: diagnostics ordering (`vibe_diagnostics`)
- Promotion mode env:
  - `VIBE_DIAGNOSTICS_SORT_MODE=selfhost-default`
- Immediate fallback env:
  - `VIBE_SELFHOST_FORCE_HOST_FALLBACK=1`

## RC Cycle Procedure

1. Enable candidate promotion mode in gate/RC environment.
2. Run parity and determinism checks:
   - `cargo test -p vibe_diagnostics --test selfhost_shadow_ordering`
   - `cargo test -p vibe_diagnostics --test selfhost_transition_toggle promoted_diagnostics_mode_matches_host_ordering`
3. Execute fallback drill in same cycle:
   - set `VIBE_SELFHOST_FORCE_HOST_FALLBACK=1`
   - run:
     - `cargo test -p vibe_diagnostics --test selfhost_transition_toggle fallback_toggle_forces_host_mode_immediately`
4. Attach RC artifact summary and update
   `reports/v1/release_candidate_checklist.md`.

## Rollback Policy

- Any parity drift, ordering regression, or release-impacting incident triggers
  immediate fallback.
- Fallback activation requires only setting:
  - `VIBE_SELFHOST_FORCE_HOST_FALLBACK=1`
- After rollback:
  - keep component in shadow evidence mode,
  - open incident and capture drift artifacts,
  - require fresh successful RC cycle before re-promotion.

## Evidence Requirements

- Workflow `.github/workflows/v1-release-gates.yml` job
  `selfhost_m4_rc_cycle_gate`
- RC artifact: `v1-selfhost-m4-rc-cycle`
- Updated references:
  - `docs/selfhost/m4_transition_criteria.md`
  - `reports/v1/release_candidate_checklist.md`
