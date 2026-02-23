# V1 Release Notes Preview

- candidate: `v1.0.0-rc1-dryrun-local`
- date: `2026-02-22`

## Highlights

- Editor and diagnostics reliability hardening for v1 release gates
- Unsafe governance and allocation/release benchmark artifacts are emitted per build cycle
- Phase 13 operational governance docs and evidence automation landed

## Known Limitations

- Hosted RC-cycle evidence is still pending for GA promotion.
- Pilot application migration case studies are local-only until RC hosted runs complete.

## Breaking Changes

- Sendability checks now reject non-sendable values on channel send paths (`E3201`).
- Release gates require explicit unsafe review markers for unsafe blocks.

## Evidence Links

- `reports/v1/readiness_dashboard.md`
- `reports/v1/release_candidate_checklist.md`
- `docs/release/known_limitations_gate.md`
