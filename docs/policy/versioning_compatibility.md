# Versioning and Compatibility Policy

Date: 2026-02-17

## Versioning Model

VibeLang follows SemVer for toolchain releases:

- `MAJOR`: breaking language/tooling changes.
- `MINOR`: backward-compatible language/tooling features.
- `PATCH`: bug fixes and non-breaking quality improvements.

## Compatibility Guarantees (v1.x)

- Source compatibility is maintained for stable language constructs.
- Lockfile schema changes require documented migration path.
- CLI command behavior changes must include migration notes if user-visible.
- Deterministic build behavior is a compatibility contract.

## Breaking Change Process

Before a breaking change can ship:

1. Add migration guide under `docs/migrations/`.
2. Add release note entry with explicit impact and upgrade path.
3. Add conformance tests for both old and new behavior where feasible.
4. Pass release dry-run workflow with migration checks enabled.

## Compatibility Exception Cases

- Security fixes may alter behavior with emergency notice.
- Unsupported/experimental flags may change without SemVer guarantees until
  declared stable.
