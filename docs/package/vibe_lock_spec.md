# `vibe.lock` Specification (v0.1)

Date: 2026-02-17

## Purpose

`vibe.lock` records resolved dependency versions and source mode so installs are
reproducible.

## Format

```toml
version = 1

[[package]]
name = "math"
version = "1.0.0"
source = "mirror"
dependencies = {}
```

## Fields

- `version`: lockfile schema version (`1` in v0.1).
- `[[package]]`: one entry per resolved transitive package.
  - `name`: package identifier.
  - `version`: concrete resolved version.
  - `source`: source class (`mirror` for current implementation).
  - `dependencies`: resolved dependency map (`name -> resolved version`).

## Determinism Guarantees

- Package entries are emitted in stable sorted order.
- Repeated lock writes with unchanged manifest/mirror must produce byte-identical
  content.

## CLI Integration

- `vibe pkg resolve` prints resolver result without writing lock.
- `vibe pkg lock` writes `vibe.lock`.
- `vibe pkg install` writes `vibe.lock` and installs mirror content to
  `.yb/pkg/store/`.
- `vibe pkg publish` writes registry payload/index (`docs/package/registry_index_spec.md`).
- `vibe pkg audit` enforces vulnerability/license policy checks (`docs/package/security_policy.md`).
- `vibe pkg upgrade-plan` and `vibe pkg semver-check` provide semver-aware upgrade guidance (`docs/package/upgrade_guide.md`).
