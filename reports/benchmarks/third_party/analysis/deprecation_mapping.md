# Cross-Language Suite Deprecation Mapping

## Old -> New canonical paths

- `runtime/benchmarks/cross_lang/` -> `runtime/benchmarks/_DEPRECATED_cross_lang/`
- `reports/benchmarks/cross_lang/` -> `reports/benchmarks/_DEPRECATED_cross_lang/`
- `tooling/metrics/collect_cross_language_benchmarks.py` -> `tooling/metrics/collect_third_party_benchmarks.py`
- `tooling/metrics/validate_cross_language_benchmarks.py` -> `tooling/metrics/validate_third_party_benchmarks.py`
- `tooling/metrics/compare_cross_language_benchmarks.py` -> `tooling/metrics/compare_third_party_benchmarks.py`

## Canonical benchmark evidence

- `reports/benchmarks/third_party/latest/results.json`
- `reports/benchmarks/third_party/latest/summary.md`
- `reports/benchmarks/third_party/analysis/deltas/latest_delta.md`
