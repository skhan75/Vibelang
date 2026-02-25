# Third-Party Benchmark Reproducibility Runbook

## Objective

Collect comparable benchmark evidence using the canonical third-party stack:
PLB-CI + hyperfine.

## Local command sequence

```bash
# quick profile
python tooling/metrics/collect_third_party_benchmarks.py --profile quick
python tooling/metrics/validate_third_party_benchmarks.py --enforcement-mode warn

# full profile
python tooling/metrics/collect_third_party_benchmarks.py --profile full
python tooling/metrics/validate_third_party_benchmarks.py --enforcement-mode strict
```

## Delta generation

```bash
python tooling/metrics/compare_third_party_benchmarks.py \
  --baseline-results reports/benchmarks/third_party/history/<baseline>.json \
  --candidate-results reports/benchmarks/third_party/latest/results.json
```

## Notes

- `quick` is intended for change-detection speed and trend checks.
- `full` is intended for release-level evidence and strict gating.
- Raw outputs from PLB-CI and hyperfine are persisted under:
  - `reports/benchmarks/third_party/<profile>/raw/`
