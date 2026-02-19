# Source Extension Policy (v1.x)

Date: 2026-02-17

## Policy Summary

- Canonical source extension: `.yb`
- Legacy extension: `.vibe` (supported in v1.x migration window)

## Compatibility Rules

- CLI/indexer/lint/test/doc/fmt workflows must accept both `.yb` and `.vibe`.
- New scaffolding defaults to `.yb`.
- Mixed-extension same-stem files in one directory are rejected to prevent
  artifact collisions.

## Opt-In Warning Policy

Legacy usage warnings are opt-in to avoid noisy migration periods:

- Set `VIBE_WARN_LEGACY_EXT=1` to emit warnings when `.vibe` files are detected.

## Deprecation Timeline

- **v1.0**: dual support, `.yb` default for new templates.
- **v1.1**: keep dual support; adoption dashboard required in release report.
- **v1.2+**: removal can be discussed only if gates below are consistently met.

## Removal Gates (No Early Removal)

`.vibe` removal is blocked unless all gates stay green for 3 consecutive releases:

1. `.yb` adoption ratio >= 90% in tracked repositories.
2. Dual-extension parity CI pass rate >= 99%.
3. Zero open critical extension-migration regressions.
4. Published migration guide coverage verified in release checklist.
