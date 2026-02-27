# Examples Parity After Workstream 2

Generated from source-built CLI (`cargo run -p vibe_cli -- check/run ...`) on 2026-02-27.

## Summary

- Total examples: `78`
- `vibe check`: `78` pass / `0` fail
- `vibe run`: `73` pass / `5` fail

## Remaining Non-Passing Files (`vibe run`)

- Intentional runtime contract demos:
  - `examples/10_contracts_intent/68_runtime_require_failure_demo.yb`
  - `examples/10_contracts_intent/69_runtime_ensure_failure_demo.yb`
- Non-entry helper modules (expected to fail with explicit entrypoint guidance):
  - `examples/08_modules_packages/project_math/demo/math.yb`
  - `examples/08_modules_packages/project_pipeline/app/parser.yb`
  - `examples/08_modules_packages/project_pipeline/app/formatter.yb`

## Machine-Readable Artifact

- `reports/examples/parity_after_ws2_source_run.json`
