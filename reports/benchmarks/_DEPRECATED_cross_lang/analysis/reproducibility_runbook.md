# Cross-Language Benchmark Reproducibility Runbook

This runbook defines the canonical procedure for reproducing benchmark evidence.

## Preconditions

- Run on a Linux host (native Linux preferred for authoritative results).
- Ensure toolchains are installed (`gcc`, `rustc`, `go`, `python3`, `node`, `tsc`).
- Run from repository root: `vibelang/`.

## Canonical Command Sequence

1. Collect quick profile:

   `python3 tooling/metrics/collect_cross_language_benchmarks.py --profile quick`

2. Validate quick profile:

   `python3 tooling/metrics/validate_cross_language_benchmarks.py --results reports/benchmarks/cross_lang/quick/results.json`

3. Collect full profile:

   `python3 tooling/metrics/collect_cross_language_benchmarks.py --profile full`

4. Validate full profile with baseline delta section:

   `python3 tooling/metrics/validate_cross_language_benchmarks.py --results reports/benchmarks/cross_lang/full/results.json --baseline-results reports/benchmarks/cross_lang/full/results.json`

5. Validate latest (synchronized summary) with baseline delta section:

   `python3 tooling/metrics/validate_cross_language_benchmarks.py --results reports/benchmarks/cross_lang/latest/results.json --baseline-results reports/benchmarks/cross_lang/full/results.json`

6. Generate explicit baseline-vs-latest delta artifacts:

   `python3 tooling/metrics/compare_cross_language_benchmarks.py --baseline-results reports/benchmarks/cross_lang/full/results.json --candidate-results reports/benchmarks/cross_lang/latest/results.json`

## Expected Artifacts

- `reports/benchmarks/cross_lang/quick/results.json`
- `reports/benchmarks/cross_lang/quick/summary.md`
- `reports/benchmarks/cross_lang/full/results.json`
- `reports/benchmarks/cross_lang/full/summary.md`
- `reports/benchmarks/cross_lang/latest/results.json`
- `reports/benchmarks/cross_lang/latest/summary.md`
- `reports/benchmarks/cross_lang/latest/trend.json`
- `reports/benchmarks/cross_lang/latest/trend.md`
- `reports/benchmarks/cross_lang/analysis/deltas/latest_delta.json`
- `reports/benchmarks/cross_lang/analysis/deltas/latest_delta.md`

## Reproducibility Acceptance Checks

- Cross-language checksum and ops parity must pass for all cases.
- Run metadata must include host and toolchain fingerprints.
- Trend report and delta report must be generated for comparison visibility.
- Any regression claims must cite both raw JSON and markdown evidence artifacts.
