# Cross-Language Suite Deprecation Mapping

## Old -> New canonical paths

- `runtime/benchmarks/cross_lang/` -> removed (legacy suite retired)
- `reports/benchmarks/cross_lang/` -> removed (legacy reports retired)
- `tooling/metrics/collect_cross_language_benchmarks.py` -> removed, use `tooling/metrics/collect_third_party_benchmarks.py`
- `tooling/metrics/validate_cross_language_benchmarks.py` -> removed, use `tooling/metrics/validate_third_party_benchmarks.py`
- `tooling/metrics/compare_cross_language_benchmarks.py` -> removed, use `tooling/metrics/compare_third_party_benchmarks.py`

## Canonical benchmark evidence

- `reports/benchmarks/third_party/full/results.json`
- `reports/benchmarks/third_party/full/summary.md`
- `reports/benchmarks/third_party/analysis/deltas/latest_delta.md`
