# Module Migration and Compatibility Guide

This guide describes how to evolve VibeLang modules without breaking downstream
teams.

## Compatibility Baseline

- Treat `module` names as stable identities.
- Treat `pub` functions as compatibility surface.
- Keep package boundary (`<package>.*`) stable across minor releases.

## Safe Changes

- Add new `pub` functions.
- Add private helper modules/functions.
- Extend internals behind existing `pub` signatures.

## Breaking Changes

- Renaming a module path.
- Renaming/removing a `pub` function.
- Moving a symbol across package boundaries.

Apply these only in coordinated major upgrades.

## Recommended Migration Pattern

1. Add new module/function.
2. Keep old `pub` function as compatibility shim.
3. Internally forward old call path to new implementation.
4. Publish migration notes and timeline.
5. Remove shim only in a planned major cut.

## Example: Symbol Move

- Old: `module demo.math`, `pub add(a, b)`
- New: `module demo.arith`, `pub add_int(a, b)`

During migration:

- Keep `demo.math.add` as a shim that forwards to `demo.arith.add_int`.
- Update callers incrementally.
- Drop the shim in the next breaking release window.

## Release Checklist

- `vibe check` passes on migrated source tree.
- Cycle, boundary, and visibility diagnostics remain clean.
- Template and integration tests stay green.
- Migration notes are published with before/after import examples.
