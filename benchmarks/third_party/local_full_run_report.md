# VibeLang Full Local Benchmark Report (Third-Party Stack)

Date (UTC): `2026-02-25T18:19:23Z`  
Run profile: `full`  
Timestamp ID: `20260225_181923Z`  
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
  --baseline-results reports/benchmarks/third_party/history/20260225_180155Z_full_results.json \
  --candidate-results reports/benchmarks/third_party/latest/results.json
```

## Evidence artifacts

- Detailed JSON (full): `reports/benchmarks/third_party/full/results.json`
- Detailed JSON (latest): `reports/benchmarks/third_party/latest/results.json`
- Timestamped snapshot: `reports/benchmarks/third_party/history/20260225_181923Z_full_results.json`
- Human-readable summary: `reports/benchmarks/third_party/latest/summary.md`
- Timestamped detailed summary: `reports/benchmarks/third_party/analysis/20260225_181923Z_detailed_summary.md`
- Latest delta: `reports/benchmarks/third_party/analysis/deltas/latest_delta.md`
- Timestamped delta: `reports/benchmarks/third_party/analysis/deltas/20260225_181951Z_delta.md`

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

- **Runtime geomean vs C**: `0.102` (faster).
- **Runtime geomean vs C++**: `0.112` (faster).
- **Runtime geomean vs Go**: `0.037` (faster).
- **Runtime geomean vs Python**: `0.005` (faster).
- **Runtime geomean vs Elixir**: `0.004` (faster).
- **Compile cold vs Kotlin**: `0.318` (faster).
- **Compile cold vs Elixir**: `0.357` (faster).
- **Compile near-parity vs Go**: `1.004` (much closer to parity than previous run).
- **Expanded VibeLang runtime coverage**: all configured problems now produced VibeLang datapoints, including `binarytrees`, `merkletrees`, and `lru`.
- **Concurrency proxy (`coro-prime-sieve`)**: `vibelang=1.643ms`, `go=12.490ms`, `python=377.597ms`, `elixir=314.664ms`.

## Gaps and risks

- **Runtime vs Kotlin**: VibeLang is still slower on shared workloads (geomean `1.876`), with the biggest hotspot in `json-serde`.
- **Compile vs Go**: VibeLang remains marginally slower on cold compile (`1.004` ratio).
- **Matrix completeness risk**: runtime lanes missing for `rust`, `zig`, `swift`, `typescript`; compile lanes missing for `c`, `cpp`, `rust`, `zig`, `swift`, `python`, `typescript`.
- **Docker reproducibility blocker on this host**: `docker info` could not reach a healthy daemon; canonical docker-backed run was not possible here.
- **Coverage caveat**: several newly added VibeLang adapter implementations are currently workload-proxy implementations. Results are useful for directional tracking, but should not be used as final release-gate claims until adapter parity hardening is complete.

## Drift vs previous full run (`20260225_180155Z`)

- Runtime geomean improved vs `go` (`-22.08%`), `kotlin` (`-30.43%`), and `python` (`-8.34%`).
- Runtime geomean regressed slightly vs `c` (`+13.53%`), `cpp` (`+12.03%`), and `elixir` (`+11.84%`), but remains < `1.0` against those baselines.
- Compile cold improved vs `go` (`-3.79%`), with minor regressions vs `elixir` (`+0.75%`) and `kotlin` (`+1.90%`).
- Interpretation: adapter fixes increased problem coverage and improved key gaps (`kotlin`, `go`), while leaving some runtime ratios to tune.

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
