# Migration: `.vibe` to `.yb` Default (v1.0)

From: pre-v1 extension baseline
To: v1.0 canonical `.yb` scaffolding

## What Changed

- New projects now default to `.yb` via `vibe new`.
- Legacy `.vibe` remains supported across CLI/indexer/lint/test flows.
- Same-stem mixed extension in one directory is rejected to avoid artifact
  collisions.

## Required Actions

1. Prefer creating new files with `.yb`.
2. Rename old `.vibe` files when convenient, not all at once.
3. Avoid keeping both `foo.vibe` and `foo.yb` in same directory.

## Verification Commands

```sh
vibe check app/main.yb
vibe test app/
VIBE_WARN_LEGACY_EXT=1 vibe check app/main.vibe
```

## Rollback

If migration causes issues, `.vibe` files remain supported in v1.x. Keep
existing files and run with warning mode disabled until blockers are fixed.
