# Cloud Reproducibility Recipe

This recipe runs the full third-party benchmark suite in a Docker-first,
machine-reproducible setup.

## Recommended host profile

- Ubuntu 24.04 LTS
- 8+ vCPU, 16+ GB RAM, NVMe storage
- Dedicated VM (no co-tenancy burst noise if possible)
- Docker Engine with daemon access

## One-time setup

```bash
git clone https://github.com/skhan75/VibeLang.git
cd VibeLang/vibelang
```

## Preflight (required)

```bash
python3 tooling/metrics/collect_third_party_benchmarks.py --profile full --preflight-only
```

Expected:
- `status: ok`
- docker daemon check is healthy
- core binaries (`git`, `dotnet`, `hyperfine`, `vibe`) are present

## Docker-first execution

Run from repository root:

```bash
bash vibelang/benchmarks/third_party/docker/run_in_runner_container.sh
```

This builds a benchmark runner image and executes:
- PLB-CI runtime/memory/concurrency lanes
- hyperfine compile lanes
- validation summary and delta artifacts

## Outputs to collect

- `reports/benchmarks/third_party/full/results.json`
- `reports/benchmarks/third_party/full/summary.md`
- `reports/benchmarks/third_party/history/` (optional archived snapshots)
- `reports/benchmarks/third_party/analysis/deltas/latest_delta.json`
- `reports/benchmarks/third_party/analysis/deltas/latest_delta.md`

## Reproducibility checklist

- Keep VM shape and region fixed between runs.
- Keep Docker and OS image pinned to the same major version.
- Record the `timestamp_id` from `results.json`.
- Record `generated_at_utc` and host metadata from the report `environment`.
- Compare against the previous full run using `latest_delta.md`.
