# Changelog

All notable changes to this project are documented here.

## [Unreleased]

### Added

- Phase 6 ecosystem baseline:
  - `vibe new`, `vibe fmt`, `vibe doc`, and `vibe pkg` command flows.
  - Package manager foundation (`vibe.toml`, deterministic resolver, lockfile,
    offline mirror install flow).
  - Self-host seed component and conformance harness.
  - Policy docs, migration guides, release process, target governance docs.
  - Metrics collection scripts and CI workflow gates.

### Changed

- Source extension policy now treats `.yb` as canonical and `.vibe` as legacy in
  v1.x migration window.
- Default metadata and scaffold conventions favor `.yb`.

### Fixed

- Added explicit guard for same-stem mixed extension collisions.

### Migration Notes

- See `docs/migrations/v1_0_source_extension_transition.md`.
