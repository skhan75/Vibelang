# M4 Transition Criteria (Default Switch)

Date: 2026-02-21

## Purpose

Define promotion rules for switching selected self-host components from
shadow-only mode to default-on execution while keeping fast rollback available.

## Promotion Candidate (First Wave)

- Component: diagnostics ordering path
- Rationale:
  - small and deterministic surface area
  - already covered by M1/M2/M3 parity checks
  - low blast radius compared to parser/type/runtime paths

## Promotion Preconditions

All must hold before enabling default-on mode for a component:

1. M2 and M3 parity gates are passing in release workflows.
2. No unresolved parity drift artifacts for candidate component.
3. Performance budget checks are passing.
4. Owner and approver signoff is recorded in
   `docs/selfhost/component_ownership.md`.
5. RC branch includes explicit rollback instructions in
   `docs/release/selfhost_transition_playbook.md`.

## Runtime Toggle Contract (Rollback/Fallback)

Diagnostics ordering candidate is controlled by:

- Promotion toggle:
  - `VIBE_DIAGNOSTICS_SORT_MODE=selfhost-default`
- Immediate fallback toggle:
  - `VIBE_SELFHOST_FORCE_HOST_FALLBACK=1`

Behavior:

- If fallback toggle is set, host ordering path is used regardless of promotion
  setting.
- Effective mode can be inspected via
  `vibe_diagnostics::diagnostics_sort_mode_label()`.

## RC Promotion Rule

For each RC cycle where a component is promoted:

- promoted mode must complete parity gates with no regressions,
- fallback mode must be exercised and verified,
- RC evidence must be recorded in
  `reports/v1/release_candidate_checklist.md` and
  `docs/release/selfhost_transition_playbook.md`.
