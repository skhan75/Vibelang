# Third-Party Benchmark Reproducibility Runbook

## Objective

Collect comparable benchmark evidence using the canonical third-party stack:
PLB-CI + hyperfine.

## Preflight gate

```bash
python3 tooling/metrics/collect_third_party_benchmarks.py --profile full --preflight-only
```

Do not proceed unless preflight reports `status: ok`.

## Docker-first command sequence

Run from repository root:

```bash
bash vibelang/benchmarks/third_party/docker/run_in_runner_container.sh
```

## Delta generation

```bash
python tooling/metrics/compare_third_party_benchmarks.py \
  --baseline-results reports/benchmarks/third_party/history/<baseline>.json \
  --candidate-results reports/benchmarks/third_party/latest/results.json
```

## Notes

- Default docker runner profile is `full`; override with:
  - `PROFILE=quick bash vibelang/benchmarks/third_party/docker/run_in_runner_container.sh`
- Validation mode can be overridden with:
  - `VALIDATION_MODE=warn ...`
- Raw outputs from PLB-CI and hyperfine are persisted under:
  - `reports/benchmarks/third_party/<profile>/raw/`
- Cloud VM guidance is documented in:
  - `benchmarks/third_party/CLOUD_REPRODUCIBILITY.md`
