# `vibe.toml` Specification (v0.1)

Date: 2026-02-17

## Purpose

`vibe.toml` defines package identity and dependency requirements for deterministic
resolution.

## Required Shape

```toml
[package]
name = "my_pkg"
version = "0.1.0"

[dependencies]
math = "^1.0.0"
```

## Rules

- `[package]` is required.
- `package.name` is required and should be unique inside a workspace/mirror.
- `package.version` uses SemVer (`major.minor.patch`).
- `[dependencies]` is optional.
- Dependency values must be valid SemVer requirements.

## Determinism

- Resolver evaluates package names in sorted order.
- Candidate versions are sorted descending, with deterministic backtracking.
- Lockfile output order is stable across runs.

## Mirror Layout (Filesystem-first)

Resolver currently expects a local mirror with this shape:

```txt
<mirror>/
  <pkg-name>/
    <version>/
      vibe.toml
      ... package sources ...
```

Remote registries remain optional and out of v0.1 compile path.
