# V1 Self-Host Readiness Report

Date: 2026-02-21  
Track: Phase 9 M1->M4 Progressive Transition

## Scope

- Maintain component-level parity evidence from M1 through M4.
- Keep host fallback controls active for promoted components.
- Ensure ownership + signoff for each promoted or shadow component.

## Overall Status

- Overall readiness state: `m4-rc-cycle-validated`
- Release posture: `shadow-first with one promoted RC candidate`
- Blocking gate set:
  - `selfhost_readiness_gate`
  - `selfhost_m2_gate`
  - `selfhost_m3_shadow_gate`
  - `selfhost_m4_rc_cycle_gate`

## Component Matrix (M1/M2/M3/M4)

| Stage | Component | Mode | Parity Counter (Required/Observed) | Owner | Signoff State |
| --- | --- | --- | --- | --- | --- |
| M1 | `formatter_core` | shadow-conformance | `30 / 2` | `@fmt-owner` | active |
| M2 | `docs_formatter_core` | shadow-parity | `10 / 2` | `@tooling-owner` | active |
| M2 | `diagnostics_formatter_core` | shadow-parity | `10 / 2` | `@diag-owner` | active |
| M3 | `parser_diag_normalization` | shadow-parity | `10 / 2` | `@frontend-owner` | active |
| M3 | `type_diag_ordering` | shadow-parity | `10 / 2` | `@types-owner` | active |
| M3 | `mir_formatting` | shadow-parity | `10 / 2` | `@mir-owner` | active |
| M4 | `diagnostics_ordering` | promoted-rc (`selfhost-default`) | `2 / 2` | `@diag-owner` | rc-approved |

## Evidence Commands (Latest Local Gate-Equivalent Dry-Run)

- `cargo test -p vibe_fmt --test selfhost_conformance`
- `cargo run -q -p vibe_cli -- test selfhost/formatter_core.yb`
- `cargo test -p vibe_doc --test selfhost_conformance`
- `cargo test -p vibe_diagnostics --test selfhost_formatter_conformance`
- `cargo test -p vibe_cli --test selfhost_m3_expansion`
- `cargo run -q -p vibe_cli -- test selfhost/frontend_shadow_slices.yb`
- `VIBE_DIAGNOSTICS_SORT_MODE=selfhost-default cargo test -p vibe_diagnostics --test selfhost_shadow_ordering`
- `VIBE_DIAGNOSTICS_SORT_MODE=selfhost-default VIBE_SELFHOST_FORCE_HOST_FALLBACK=1 cargo test -p vibe_diagnostics --test selfhost_transition_toggle fallback_toggle_forces_host_mode_immediately`

## Safety Controls

- Promoted diagnostics ordering toggle:
  - `VIBE_DIAGNOSTICS_SORT_MODE=selfhost-default`
- Immediate fallback override:
  - `VIBE_SELFHOST_FORCE_HOST_FALLBACK=1`
- Fallback drill evidence:
  - `crates/vibe_diagnostics/tests/selfhost_transition_toggle.rs`
- Operational policy:
  - `docs/selfhost/m4_transition_criteria.md`
  - `docs/release/selfhost_transition_playbook.md`
  - `docs/selfhost/component_ownership.md`

## Go / No-Go Snapshot

- M1/M2/M3 parity posture: `go` (shadow gates passing)
- M4 promoted RC candidate posture: `go` (promotion + fallback drill validated)
- GA posture: `conditional-go` (depends on non-selfhost release blockers outside Phase 9)
