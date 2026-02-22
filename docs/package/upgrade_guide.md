# Dependency Upgrade Guide (Phase 12)

## Commands

- Upgrade plan:

```bash
vibe pkg upgrade-plan --path <project-root> --mirror <mirror-root>
```

- SemVer delta classification:

```bash
vibe pkg semver-check --current <version> --next <version>
```

## Interpreting upgrade plan output

- `latest_compatible`: newest version that satisfies current manifest constraint.
- `latest_available`: newest version in mirror/registry.
- `manifest_change=yes`: upgrading to latest available requires manifest SemVer range change.

## Recommended flow

1. Inspect `upgrade-plan` results.
2. For `manifest_change=yes`, run `semver-check` on intended target and review breaking risks.
3. Update `vibe.toml`, regenerate lockfile, run tests/audit, then publish.
