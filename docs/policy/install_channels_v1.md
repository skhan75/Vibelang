# Install Channels Policy (v1)

Date: 2026-02-21

## Channels

- `stable`: production-ready tagged releases
- `rc`: release candidates used for final validation
- `nightly` (optional): integration snapshots for early adopters

## Channel Guarantees

| Channel | Stability | Signed Artifacts | Install Smoke Coverage |
| --- | --- | --- | --- |
| stable | highest | required | required on all tier-1 targets |
| rc | pre-GA candidate | required | required on all tier-1 targets |
| nightly | best-effort | recommended | recommended |

## Promotion Rules

- `rc -> stable` promotion requires all blocking v1 gates green, including
  independent install and distribution security controls.
- `nightly -> rc` promotion requires no open `P0` in readiness dashboard for
  covered surfaces.

## Rollback

- Channel rollback must keep previous signed artifacts available.
- Rollback announcements must include affected target(s), reason, and replacement
  version.
