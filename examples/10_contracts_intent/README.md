# Contracts and Intent Examples

This folder demonstrates VibeLang annotation capabilities:

- `@intent`
- `@examples`
- `@require`
- `@ensure`
- `old(...)` and `.` return placeholder in ensure checks
- `@effect` declarations and transitive effects

## Quick commands

- Run a passing contract/example suite:
  - `vibe test examples/10_contracts_intent/63_all_annotations_combo.yb`
- Run a second passing contract/example suite:
  - `vibe test examples/10_contracts_intent/55_examples_table.yb`

## Positive examples

- `54_intent_minimal.yb` through `67_public_api_style_contracts.yb`
- `70_concurrency_effect_contracts.yb`

## Intentional runtime-failure demos

- `68_runtime_require_failure_demo.yb`
- `69_runtime_ensure_failure_demo.yb`

These two files are valid for `vibe check` but intentionally fail under `vibe run`
to showcase runtime contract enforcement behavior.
