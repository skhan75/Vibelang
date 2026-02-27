# Examples Parity After Workstream 1

Generated from source-built CLI (`cargo run -p vibe_cli -- run ...`) on 2026-02-27.

## Summary

- Total examples: `75`
- `vibe run`: `70` pass / `5` fail

## Remaining Non-Passing Files

- Intentional runtime contract demos:
  - `examples/10_contracts_intent/68_runtime_require_failure_demo.yb`
  - `examples/10_contracts_intent/69_runtime_ensure_failure_demo.yb`
- Non-entry helper modules (expected to fail with explicit entrypoint guidance):
  - `examples/08_modules_packages/project_math/demo/math.yb`
  - `examples/08_modules_packages/project_pipeline/app/parser.yb`
  - `examples/08_modules_packages/project_pipeline/app/formatter.yb`

## Machine-Readable Artifact

- `reports/examples/parity_after_ws1_source_run.json`
