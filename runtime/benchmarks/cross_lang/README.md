# Cross-Language Benchmark Suite

This suite compares VibeLang against C, Rust, Go, Python, and TypeScript using
a starter pack of eight deterministic workloads.

## Scope

The starter workloads target a balanced mix of benchmark surfaces:

- compute recursion (`fib_recursive`)
- compute + memory arithmetic (`prime_sieve`)
- list/container sorting (`sort_int`)
- map update/read pressure (`hashmap_frequency`)
- map int-key pressure (`hashmap_frequency_int_key`)
- map str-key pressure (`hashmap_frequency_str_key`)
- string construction + parse loop (`string_concat_checksum`)
- JSON utility path (`json_roundtrip`)
- channel round-trip latency (`channel_pingpong`)
- worker-pool reduction throughput (`worker_pool_reduce`)

## Layout

Each benchmark case has one implementation per language:

`cases/<case-id>/vibelang/main.yb`
`cases/<case-id>/c/main.c`
`cases/<case-id>/rust/main.rs`
`cases/<case-id>/go/main.go`
`cases/<case-id>/python/main.py`
`cases/<case-id>/typescript/main.ts`

## Program Output Contract

Every benchmark binary prints exactly three trailing lines:

1. `RESULT`
2. `<checksum>`
3. `<ops>`

The collector parses this contract and validates cross-language parity by case.

## Runner

Use:

- `tooling/metrics/collect_cross_language_benchmarks.py`
- `tooling/metrics/validate_cross_language_benchmarks.py`
- `tooling/metrics/compare_cross_language_benchmarks.py`

The collector emits:

- `reports/benchmarks/cross_lang/latest/results.json`
- `reports/benchmarks/cross_lang/latest/summary.md`

Delta reports can be generated between a locked baseline and a candidate run:

- `reports/benchmarks/cross_lang/analysis/deltas/<timestamp>_delta.json`
- `reports/benchmarks/cross_lang/analysis/deltas/<timestamp>_delta.md`
- `reports/benchmarks/cross_lang/analysis/deltas/latest_delta.json`
- `reports/benchmarks/cross_lang/analysis/deltas/latest_delta.md`

Profile drift/trend artifacts are emitted when both quick and full results are present:

- `reports/benchmarks/cross_lang/latest/trend.json`
- `reports/benchmarks/cross_lang/latest/trend.md`

Reproducibility procedure:

- `reports/benchmarks/cross_lang/analysis/reproducibility_runbook.md`

Budget and triage controls:

- `reports/benchmarks/cross_lang/analysis/performance_budgets.json`
- `reports/benchmarks/cross_lang/analysis/regression_triage_template.md`
- `reports/benchmarks/cross_lang/analysis/rollback_protocol.md`

