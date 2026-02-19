# VibeLang Checks Runner (v0.1)

## Purpose

`checks_runner` executes generated contract checks and examples in local and CI pipelines.

## Inputs

- Compiled check metadata from contract engine
- Generated test IR/module
- Build profile (`dev`, `test`, `release`)

## Execution Steps

1. Load generated checks module.
2. Execute precondition/postcondition check suite.
3. Execute generated `@examples` tests.
4. Aggregate failures with source locations.
5. Emit machine-readable and human-readable reports.

## CLI Design

- `vibe test <path>` run deterministic checks/examples for a file or directory
- `vibe checks` run all checks for workspace
- `vibe checks --file app/math.yb` scoped run
- `vibe checks --changed` run checks for changed files only
- `vibe checks --format json` output for CI annotation

## Deterministic Utility APIs

Contract/example execution should use deterministic helpers only (`len`, `min`, `max`, `sorted_desc`, `sort_desc`, `take`) so repeated runs are stable across machines.

## Output Format (Human)

```txt
checks: 42 passed, 1 failed, 0 skipped
failed:
  topK @ensure len(.) == min(k, len(xs))
  at app/math.yb:21:3
```

## Output Format (JSON)

```txt
{
  "total": 43,
  "passed": 42,
  "failed": 1,
  "failures": [
    {
      "function": "topK",
      "contract": "@ensure len(.) == min(k, len(xs))",
      "file": "app/math.yb",
      "line": 21,
      "column": 3
    }
  ]
}
```

## Performance Targets

- Check execution overhead in local dev: under 2 seconds for medium service module set.
- Changed-file check mode: under 300 ms median for single-file edits.

## CI Integration

- `vibe checks` is mandatory stage for protected branches.
- Contract coverage metric published as CI artifact.
