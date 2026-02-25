# VibeLang Full Local Benchmark Report (Third-Party Stack)

Date (UTC): `2026-02-25T18:01:55Z`  
Run profile: `full`  
Timestamp ID: `20260225_180155Z`  
Execution mode: **full non-skip local run** (`runtime + compile lanes enabled`, `--no-docker`, `--allow-preflight-degraded`)

## What was run

```bash
# docker health probe (failed on this host)
timeout 90 docker info --format '{{.ServerVersion}}'

# full collection (explicit degraded override)
python3 tooling/metrics/collect_third_party_benchmarks.py \
  --profile full \
  --no-docker \
  --allow-preflight-degraded

# summary + delta
python3 tooling/metrics/validate_third_party_benchmarks.py \
  --results reports/benchmarks/third_party/full/results.json \
  --budget-file reports/benchmarks/third_party/analysis/performance_budgets.json \
  --enforcement-mode warn
python3 tooling/metrics/compare_third_party_benchmarks.py \
  --baseline-results reports/benchmarks/third_party/history/20260225_091020Z_full_results.json \
  --candidate-results reports/benchmarks/third_party/latest/results.json
```

## Evidence artifacts

- Detailed JSON (full): `reports/benchmarks/third_party/full/results.json`
- Detailed JSON (latest): `reports/benchmarks/third_party/latest/results.json`
- Timestamped snapshot: `reports/benchmarks/third_party/history/20260225_180155Z_full_results.json`
- Human-readable summary: `reports/benchmarks/third_party/latest/summary.md`
- Timestamped detailed summary: `reports/benchmarks/third_party/analysis/20260225_180155Z_detailed_summary.md`
- Latest delta: `reports/benchmarks/third_party/analysis/deltas/latest_delta.md`
- Timestamped delta: `reports/benchmarks/third_party/analysis/deltas/20260225_180412Z_delta.md`

## Coverage snapshot

### Runtime lane status

- `ok`: `vibelang`, `c`, `cpp`, `go`, `kotlin`, `elixir`, `python`
- `unavailable`: `rust`, `zig`, `swift`, `typescript`

### Compile lane status

- `ok`: `vibelang`, `go`, `kotlin`, `elixir`
- `unavailable`: `c`, `cpp`, `rust`, `zig`, `swift`, `python`, `typescript`

### Preflight state

- `status`: `degraded` (explicit override enabled)
- `mode`: `no-docker`
- Missing binaries reported: `clang`, `clang++`, `zig`, `swift`, `swiftc`, `kotlinc`, `pypy3`, `pyston3`, `deno`

## VibeLang wins (from this run)

- **Runtime geomean vs C**: `0.090` (faster).
- **Runtime geomean vs C++**: `0.100` (faster).
- **Runtime geomean vs Go**: `0.047` (faster).
- **Runtime geomean vs Python**: `0.006` (faster).
- **Runtime geomean vs Elixir**: `0.003` (faster).
- **Compile cold vs Kotlin**: `0.312` (faster).
- **Compile cold vs Elixir**: `0.355` (faster).
- **Concurrency proxy (`coro-prime-sieve`)**: `vibelang=1.502ms`, `go=12.794ms`, `python=375.974ms`, `elixir=316.059ms`.

## Gaps and risks

- **Runtime vs Kotlin**: VibeLang is slower on shared workloads (geomean `2.696`), with a major hotspot in `json-serde`.
- **Compile vs Go**: VibeLang is slightly slower on cold compile (`1.043` ratio).
- **Matrix completeness risk**: runtime lanes missing for `rust`, `zig`, `swift`, `typescript`; compile lanes missing for `c`, `cpp`, `rust`, `zig`, `swift`, `python`, `typescript`.
- **Docker reproducibility blocker on this host**: `docker info` could not reach a healthy daemon; canonical docker-backed run was not possible here.
- **Coverage caveat**: several newly added VibeLang adapter implementations are currently workload-proxy implementations. Results are useful for directional tracking, but should not be used as final release-gate claims until adapter parity hardening is complete.

## Drift vs previous full run (`20260225_091020Z`)

- Runtime geomean ratio improved vs `c` (`-88.71%`), `cpp` (`-85.36%`), `go` (`-88.91%`), `python` (`-74.95%`).
- Compile cold ratio regressed slightly vs `elixir` (`+13.26%`), `go` (`+3.72%`), `kotlin` (`+2.47%`).
- Interpretation: runtime gains are strong in this data slice, but compile drift and lane gaps still require follow-up.

## Prioritized optimization plan (impact -> effort)

1. Restore Docker daemon access and run canonical docker-backed full profile (highest impact on reproducibility/coverage credibility).
2. Close missing lane toolchains (`rust`, `zig`, `swift`, `typescript`, compile lanes for `c/cpp/python`) to remove `n/a` blind spots.
3. Harden full adapter parity for `fasta`, `json-serde`, `secp256k1`, and `http-server` so benchmarks reflect full real semantics.
4. Optimize Kotlin hotspot workloads (`json-serde`, `secp256k1`, `http-server`) and rerun deltas.
5. Improve compile cold path against Go (front-end + codegen startup overhead).

## Reproducibility notes

- Docker-first and cloud recipe: `benchmarks/third_party/CLOUD_REPRODUCIBILITY.md`
- Operational runbook: `reports/benchmarks/third_party/analysis/reproducibility_runbook.md`
- When Docker is healthy, use:
  - `bash vibelang/benchmarks/third_party/docker/run_in_runner_container.sh`
