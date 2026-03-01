# VibeLang Third-Party Benchmark Report

This report documents the **strict Docker-backed** PLB-CI run used for the canonical artifacts:

- `reports/benchmarks/third_party/full/results.json`
- `reports/benchmarks/third_party/full/summary.md`

Date (UTC): `2026-03-01T07:28:22Z`  
Run profile: `full`  
Publication mode: **strict** (`--publication-mode`)  
Execution mode: **docker-first runner container** (reproducible toolchains, pinned PLB-CI ref)

Publication classification: **publication-mode compliant** (strict validation passed with allowlisted lane gaps).

## What changed before this run (benchmarks unblock work)

- Bench-only runtime was split behind the `bench-runtime` feature and exposed under `bench.*`.
- PLB-CI adapter parity is now canonical in publication mode.
- The third-party runner container installs required toolchains and builds a bench-enabled `vibe`.

## Commands run

```bash
# docker-first strict run (runner image must exist)
docker build -f benchmarks/third_party/docker/Dockerfile -t vibelang-third-party-bench:local .
docker run --rm \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v "$(pwd):/workspace/VibeStack" \
  -w /workspace/VibeStack/vibelang \
  -e PROFILE=full \
  -e VALIDATION_MODE=strict \
  vibelang-third-party-bench:local

# local re-validation (must pass)
python3 tooling/metrics/validate_third_party_benchmarks.py --enforcement-mode strict
python3 tooling/metrics/validate_adapter_parity.py \
  --manifest benchmarks/third_party/plbci/adapters/vibelang/PARITY_MANIFEST.yaml \
  --matrix-file benchmarks/third_party/plbci/config/language_matrix.json \
  --adapter-root benchmarks/third_party/plbci/adapters/vibelang \
  --publication-mode
```

## Evidence artifacts (newly generated)

- Detailed JSON (canonical): `reports/benchmarks/third_party/full/results.json`
- Human-readable summary (canonical): `reports/benchmarks/third_party/full/summary.md`
- Delta report (self-delta, for convenience): `reports/benchmarks/third_party/analysis/deltas/latest_delta.md`

## Coverage snapshot

### Runtime lane status

- `ok`: `vibelang`, `c`, `cpp`, `rust`, `go`, `zig`, `kotlin`, `elixir`, `python`, `typescript`
- `unavailable` (allowlisted): `swift`

### Compile lane status

- `ok`: `vibelang`, `c`, `cpp`, `rust`, `go`, `zig`, `python`, `typescript`
- `unavailable` (allowlisted): `swift`, `kotlin`, `elixir`

### Preflight state

- `status`: `ok` (publication-mode requires this)
- Docker enabled: `true`
- PLB-CI ref pinned: `ad18b203dd1769724f4eea94fc3ac1e99f6593e0`

## Easy-to-read result summary

### Key wins

- Runtime geomean faster than:
  - `c` (`0.549x`)
  - `cpp` (`0.371x`)
  - `rust` (`0.485x`)
  - `zig` (`0.398x`)
  - `elixir` (`0.028x`)
  - `python` (`0.224x`)
  - `typescript` (`0.066x`)
- Compile cold faster than:
  - `rust` (`0.566x`)
  - `zig` (`0.437x`)

### Main performance gaps

- Runtime slower than:
  - `kotlin` (`17.158x`)
  - `go` (`13.220x`)
- Compile cold slower than:
  - `c` (`1.718x`)
  - `cpp` (`1.708x`)
  - `python` (`1.427x`)
  - `typescript` (`1.083x`)
  - `go` (`1.056x`)

### Concurrency proxy snapshot (`coro-prime-sieve`, lower is better)

- `vibelang`: `3.349 ms`
- `kotlin`: `1.285 ms`
- `go`: `2.631 ms`
- `typescript`: `133.574 ms`
- `python`: `153.959 ms`
- `elixir`: `329.597 ms`

## Memory footprint and binary size

The canonical JSON includes:

- runtime memory samples (`mem_bytes`) aggregated as `memory_mean_bytes` in `reports/benchmarks/third_party/full/summary.md`
- compile-lane `binary_size_bytes` (when the compile lane is available)

Snapshot (from the strict run at `2026-03-01T07:28:22Z`):

| language | memory_mean_bytes | binary_size_bytes |
| --- | ---: | ---: |
| vibelang | 33273669 | 249280 |
| c | 3670528 | 16200 |
| cpp | 2031616 | 16200 |
| rust | 8055286 | 316248 |
| go | 1966080 | 1704 |
| zig | 3732593 | 1711056 |
| python | 27168426 | 1077 |
| typescript | 78215680 | 92332880 |
| elixir | 83461266 | n/a |

## Publication notes (strict mode)

- Adapter parity: **pass** in publication mode (0 noncanonical problems).
- Strict validation: **pass** (see `reports/benchmarks/third_party/full/summary.md` “Budget Gate Output” section).
- Allowlisted lane gaps:
  - `swift` runtime+compile unavailable (allowlisted)
  - `kotlin` + `elixir` compile lanes unavailable (allowlisted)

## Action checklist

- Canonical checklist for remaining work: `reports/benchmarks/third_party/analysis/gaps_optimization_blocker_checklist.md`
