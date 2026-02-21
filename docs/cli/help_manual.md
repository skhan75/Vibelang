# Vibe CLI Help Manual

Status: normative user-facing CLI help contract for v1+.

## Global Help Entry Points

- `vibe --help`
- `vibe -h`
- `vibe help`
- `vibe help <command>`
- `vibe <command> --help`

## Root Help Contract

`vibe --help` MUST include:

- command summary list
- global options section
- exit code semantics
- runnable examples

The root help output is intended as the quick manual and discovery surface for
new users.

## Per-Command Help Contract

Each command help output MUST include:

- `USAGE`
- short `DESCRIPTION` and/or command intent
- supported flags and accepted values
- special constraints (for example `vibe test` and `vibe run` rejecting
  `--emit-obj-only`)

Supported command help pages:

- `check`
- `ast`
- `hir`
- `mir`
- `build`
- `run`
- `test`
- `index`
- `lsp`
- `fmt`
- `doc`
- `new`
- `pkg`
- `lint`

## Error Guidance Rules

- Unknown commands should return usage guidance and suggest running
  `vibe --help`.
- Invalid `help`/`--help` arity should return command-specific usage text.

## Exit Behavior

- Help requests return exit code `0`.
- Usage/argument errors return exit code `2`.

## Regression Coverage

Help behavior is regression-tested in:

- `crates/vibe_cli/tests/cli_help_snapshots.rs`
