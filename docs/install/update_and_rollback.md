# Packaged Install Update and Rollback

## Update Procedure

1. Download next artifact for your platform.
2. Verify checksum/signature/provenance as in platform install docs.
3. Extract to versioned install path.
4. Point `PATH` (or symlink) to new version directory.
5. Run `vibe --version` and `vibe run` hello-world smoke.

## Rollback Procedure

1. Keep at least one previous verified package in local cache.
2. Re-point `PATH`/symlink to previous version directory.
3. Re-run verification smoke.
4. Record rollback reason in release notes/incident channel.

## Operational Guidance

- Do not rollback to unsigned/unverified artifacts.
- Prefer channel-aware rollback (stable->previous stable, rc->previous rc).
- If rollback is triggered by signature/provenance failures, treat as release
  blocker and halt further promotion.
