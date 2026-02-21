# Selfhost Component Ownership Matrix

Date: 2026-02-21

## Purpose

Define owner/accountability boundaries and signoff requirements for M1-M4
self-host components participating in release gates.

## Ownership Matrix

| Stage | Component | Primary Owner | Backup Owner | Gate(s) | Signoff Requirement |
| --- | --- | --- | --- | --- | --- |
| M1 | `formatter_core` | `@fmt-owner` | `@tooling-owner` | `selfhost_readiness_gate` | parity + determinism pass |
| M2 | `docs_formatter_core` | `@tooling-owner` | `@fmt-owner` | `selfhost_m2_gate` | fixture parity pass |
| M2 | `diagnostics_formatter_core` | `@diag-owner` | `@tooling-owner` | `selfhost_m2_gate` | fixture parity pass |
| M3 | `parser_diag_normalization` | `@frontend-owner` | `@diag-owner` | `selfhost_m3_shadow_gate` | parity + drift artifact pass |
| M3 | `type_diag_ordering` | `@types-owner` | `@diag-owner` | `selfhost_m3_shadow_gate` | parity + drift artifact pass |
| M3 | `mir_formatting` | `@mir-owner` | `@frontend-owner` | `selfhost_m3_shadow_gate` | parity + drift artifact pass |
| M4 | `diagnostics_ordering` | `@diag-owner` | `@release-owner` | `selfhost_m4_rc_cycle_gate`, `selfhost_transition_gate` | promoted mode + fallback drill pass |

## Incident and Rollback Routing

- P0/P1 parity regressions: page component primary owner and release owner.
- Immediate mitigation for promoted diagnostics ordering:
  - set `VIBE_SELFHOST_FORCE_HOST_FALLBACK=1`
- Post-incident requirements:
  - attach drift artifact bundle,
  - update RC checklist,
  - obtain owner re-signoff before re-promotion.

## Release Signoff Checklist

Before release branch merge, each component marked `promoted-rc` or
`default-selfhost` must have:

- owner signoff in release PR description,
- passing gate artifacts linked,
- rollback command documented,
- incident contact path confirmed.
