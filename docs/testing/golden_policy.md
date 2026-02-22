# Golden Snapshot Policy (Phase 12)

## Purpose

Ensure diagnostic and snapshot updates stay explicit, reviewable, and deterministic.

## Tooling entrypoint

Use the dedicated updater instead of ad-hoc shell env usage:

```bash
python3 tooling/golden/update_goldens.py --suite frontend
python3 tooling/golden/update_goldens.py --suite phase12
python3 tooling/golden/update_goldens.py --suite all
```

The tool sets `UPDATE_GOLDEN=1` for the selected suites and fails fast on the first failing
command.

## Review requirements

- Golden updates must be committed with corresponding implementation changes.
- PR description should explain why snapshots changed.
- Unexpected broad diffs require manual root-cause review before merge.

## Determinism guard

- Re-running golden updates without source changes should produce no diff.
- Any nondeterministic golden churn is release-blocking until explained/fixed.
