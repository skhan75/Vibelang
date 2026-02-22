# Editor and CI Consistency Model

Date: 2026-02-22

## Goal

Prevent drift between diagnostics and behavior seen in local editor workflows and
CI validation pipelines.

## Consistency gates

Phase 13.1 introduces explicit gates:

1. JSON-RPC protocol smoke (`tooling/phase13/protocol_smoke.py`)
2. Diagnostics parity check (`tooling/phase13/check_diagnostics_parity.py`)
3. Editor UX benchmark/budget check (`tooling/phase13/benchmark_editor_ux.py --enforce`)
4. Budget policy validation (`tooling/metrics/validate_v1_quality_budgets.py`)

These are orchestrated by workflow:

- `.github/workflows/phase13-editor-ux.yml`

## Diagnostics parity contract

For a fixed fixture input, CLI and LSP diagnostics must expose equivalent error
code sets.

Current parity artifact:

- `reports/phase13/editor_ci_consistency.json`

## Performance consistency contract

LSP/editor latency and index memory budgets are tracked in:

- `reports/v1/quality_budgets.json` (`editor_ux_benchmarks`)

Collected metrics artifact:

- `reports/phase13/editor_ux_metrics.json`

## Local reproducibility commands

```bash
python3 tooling/phase13/protocol_smoke.py
python3 tooling/phase13/check_diagnostics_parity.py
python3 tooling/phase13/benchmark_editor_ux.py --enforce
VIBE_REQUIRE_EDITOR_UX_METRICS=1 python3 tooling/metrics/validate_v1_quality_budgets.py
```

