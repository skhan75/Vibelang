# Vibe CLI Version Output Policy

Status: normative output contract for `vibe --version`.

## Supported Invocation Forms

- Human-readable:
  - `vibe --version`
  - `vibe version`
- Machine-readable:
  - `vibe --version --json`

## Human-Readable Contract

The human-readable format is:

`vibe <semver> (commit=<sha-or-unknown>, target=<arch-os>, profile=<dev|release>)`

Required fields:

- semantic version (`<semver>`)
- build commit (`commit`)
- build target (`target`)
- build profile (`profile`)

## JSON Contract

`vibe --version --json` emits a single JSON object with stable keys:

- `name`
- `version`
- `commit`
- `target`
- `profile`

Example shape:

```json
{"name":"vibe","version":"0.1.0","commit":"unknown","target":"x86_64-linux","profile":"release"}
```

## Stability Rules

- Output must be deterministic for repeated invocations in the same binary.
- Unknown commit metadata is represented as `"unknown"` instead of omitting the
  key.
- Additional keys may be added only with explicit compatibility note in release
  docs.

## Error Behavior

- `vibe --version` accepts at most one extra flag: `--json`.
- Unknown extra arguments return usage guidance and exit with code `2`.

## Regression Coverage

Version behavior is regression-tested in:

- `crates/vibe_cli/tests/cli_version.rs`
