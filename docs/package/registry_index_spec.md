# Registry Index Specification (Phase 12)

## Purpose

Define deterministic on-disk index format used by `vibe pkg publish`.

## Location

Default registry root:

```txt
<project>/.yb/pkg/registry/
  index.toml
  <package>/<version>/...
```

## Format (`index.toml`)

```toml
version = 1

[[entry]]
name = "demo"
version = "0.1.0"
source = "local"
```

## Rules

- `version` is index schema version (currently `1`).
- `entry` rows are sorted by package name then semantic version.
- Duplicate `name + version` entries are rejected.
- Publish payload mirrors package source tree excluding runtime/build metadata directories.

## Determinism

- Repeated publishes with unchanged package state produce stable index ordering and payload layout.
