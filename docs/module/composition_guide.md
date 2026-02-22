# VibeLang Module Composition Guide

This guide defines the v1.0 project-scale module rules for deterministic
multi-file applications.

## Canonical Layout

- Use explicit module headers in composed projects:
  - `module <package>.<path_segments>`
- Module names must match file layout relative to project root:
  - `demo/main.yb` -> `module demo.main`
  - `demo/http/router.yb` -> `module demo.http.router`
- Entry files that use `import` must also declare `module`.

## Import and Package Boundaries

- Imports are explicit: `import demo.http.router`
- Import targets must exist in project sources.
- Cross-package imports are rejected in v1:
  - if entry package is `demo`, `import otherpkg.util` is an error.
- Import cycles are rejected; diagnostics include the cycle path.

## Visibility

- Functions are module-private by default.
- Mark exported declarations with `pub`.
- Calling private declarations from imported modules produces an error.

## Recommended Layouts

## Service

- `service_name/main.yb` (entry)
- `service_name/router.yb`
- `vibe.toml`

## CLI

- `tool_name/main.yb` (entry)
- `tool_name/commands.yb`
- `vibe.toml`

## Library

- `lib.yb`
- `vibe.toml`

## Validation Commands

- `vibe check <entry>`
- `vibe run <entry>`
- `vibe test <entry_or_dir>`

Use these commands in CI to keep module graphs deterministic and visible.
