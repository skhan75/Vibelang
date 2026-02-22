# Package Publishing Guide (Phase 12)

## Publish command

```bash
vibe pkg publish --path <project-root> [--registry <registry-root>]
```

## Behavior

- Validates package manifest and semantic version.
- Verifies package has source files (`.yb`/`.vibe`).
- Copies publishable project contents into registry layout.
- Updates deterministic `index.toml`.
- Rejects duplicate publishes for existing package/version.

## Recommended workflow

1. `vibe pkg lock --path <project-root>`
2. `vibe pkg install --path <project-root> --mirror <mirror-root>`
3. `vibe pkg audit --path <project-root> --mirror <mirror-root> --policy <policy> --advisory-db <db>`
4. `vibe pkg publish --path <project-root> --registry <registry-root>`

## Notes

- Current publish flow is local-filesystem first.
- Hosted registry API integration remains out-of-scope for this phase.
