# V1 Selfhost M3 Expansion Report

Date: 2026-02-21

## Objective

Track expanded M3 frontend shadow slices and enforce deterministic host-vs-shadow
parity with drift artifacts.

## Expanded Shadow Slice Scope

- Parser diagnostics normalization
  - fixture: `compiler/tests/fixtures/parse_err/multiple_errors.vibe`
  - shadow output: `selfhost/fixtures/m3_parser_diag_normalization.selfhost.out`
- Type diagnostics ordering
  - fixture: `compiler/tests/fixtures/type_err/map_set_value_mismatch.yb`
  - shadow output: `selfhost/fixtures/m3_type_diag_ordering.selfhost.out`
- MIR formatting
  - fixture: `compiler/tests/fixtures/snapshots/pipeline_sample.vibe`
  - shadow output: `selfhost/fixtures/m3_mir_formatting.selfhost.out`

## Parity and Determinism Enforcement

- Host-vs-shadow parity harness:
  - `cargo test -p vibe_cli --test selfhost_m3_expansion`
- Self-host M3 shadow executable contract:
  - `cargo run -q -p vibe_cli -- test selfhost/frontend_shadow_slices.yb`
- Determinism checks:
  - `host_shadow_outputs_match_m3_slice_fixtures`
  - `m3_slice_repeat_runs_are_deterministic`

## CI Gate and Drift Artifacts

- Blocking gate:
  - `.github/workflows/v1-release-gates.yml` job `selfhost_m3_shadow_gate`
- Drift artifact bundle:
  - uploaded as `v1-selfhost-m3-shadow`
  - includes `/tmp/v1-selfhost-m3` per-slice `host.out`, `selfhost.out`,
    `diff.txt`, `status.txt`

## Shadow Performance Budgets

Budget source: `crates/vibe_cli/tests/selfhost_m3_expansion.rs` test
`m3_shadow_performance_budgets_are_within_thresholds`.

- CI thresholds (from `selfhost_m3_shadow_gate`):
  - `VIBE_M3_PERF_LOOPS=20`
  - `VIBE_M3_MAX_LATENCY_OVERHEAD_PCT=400`
  - `VIBE_M3_MAX_MEMORY_OVERHEAD_BYTES=32768`
- Latest local gate-equivalent metrics (`/tmp/v1-selfhost-m3/performance_metrics.json`):
  - `baseline_ms=3.274`
  - `shadow_ms=3.119`
  - `latency_overhead_pct=-4.729`
  - `baseline_peak_bytes=468`
  - `shadow_peak_bytes=951`
  - `memory_overhead_bytes=483`

## Current Status

- Expanded M3 slice coverage: `complete`
- Host/self-host shadow dual-path parity checks: `validated`
- Drift artifact emission on every gate run: `active`
- Shadow performance budget enforcement: `validated`
