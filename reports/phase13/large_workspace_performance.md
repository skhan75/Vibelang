# Phase 13.1 Large Workspace Performance Report

Date: 2026-02-22

## Source

Metrics were collected by:

- `tooling/phase13/benchmark_editor_ux.py --enforce`

Artifacts:

- `reports/phase13/editor_ux_metrics.json`

## Collected metrics snapshot

- `index_cold_ms`: `43`
- `index_incremental_ms`: `1`
- `index_memory_bytes`: `154673`
- `lsp_initialize_ms`: `105`
- `lsp_did_open_ms`: `0`
- `lsp_completion_ms`: `0`
- `lsp_formatting_ms`: `0`
- `lsp_shutdown_ms`: `0`

## Budget policy

Budget thresholds are defined in:

- `reports/v1/quality_budgets.json` section `editor_ux_benchmarks`

Validation command:

```bash
VIBE_REQUIRE_EDITOR_UX_METRICS=1 python3 tooling/metrics/validate_v1_quality_budgets.py
```

Result: pass.

