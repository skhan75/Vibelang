# VibeLang Full Local Benchmark Report (Third-Party Stack)

Date (UTC): `2026-02-26T07:21:50Z`  
Run profile: `full`  
Timestamp ID: `20260226_072150Z`  
Execution mode: **full local degraded run** (`runtime + compile lanes enabled`, `--no-docker`, `--allow-preflight-degraded`)

Publication classification: **internal-only** (not strict apples-to-apples publishable yet).

## What changed before this run

- Old generated benchmark reports were removed from `reports/benchmarks/third_party/` and regenerated from scratch.
- Installed/added local toolchains for better lane coverage:
  - `zig 0.15.2`
  - `deno 2.7.1`
  - `kotlin/kotlinc 2.3.10`
  - `pypy3 7.3.20`
  - `pyston3 2.3.5`
  - `clang`/`clang++` now available via Zig's Clang frontend.
- Remaining toolchain gap: `swift` / `swiftc` (Swiftly installation attempt did not complete in this environment).

## Commands run

```bash
# strict attempt (expected to fail until remaining canonical blockers are fixed)
python3 tooling/metrics/collect_third_party_benchmarks.py --profile full --publication-mode

# full benchmark collection (fresh, after clearing old generated reports)
python3 tooling/metrics/collect_third_party_benchmarks.py \
  --profile full \
  --no-docker \
  --allow-preflight-degraded

# summary validation + strict gate check
python3 tooling/metrics/validate_third_party_benchmarks.py
python3 tooling/metrics/validate_third_party_benchmarks.py --publication-mode
```

## Evidence artifacts (newly generated)

- Detailed JSON (canonical): `reports/benchmarks/third_party/full/results.json`
- Human-readable summary (canonical): `reports/benchmarks/third_party/full/summary.md`

## Coverage snapshot

### Runtime lane status

- `ok`: `vibelang`, `c`, `cpp`, `go`, `kotlin`, `elixir`, `python`, `typescript`
- `unavailable`: `rust`, `zig`, `swift`

### Compile lane status

- `ok`: `vibelang`, `c`, `cpp`, `go`, `kotlin`, `elixir`, `python`, `typescript`
- `unavailable`: `rust`, `zig`, `swift`

### Preflight state

- `status`: `degraded`
- `mode`: `no-docker`
- Remaining preflight gaps: `swift`, `swiftc`

## Easy-to-read result summary

### Key wins

- Runtime geomean is faster than:
  - `c` (`0.093x`)
  - `cpp` (`0.095x`)
  - `go` (`0.037x`)
  - `elixir` (`0.003x`)
  - `python` (`0.010x`)
  - `typescript` (`0.014x`)
- Compile cold is faster than:
  - `kotlin` (`0.305x`)
  - `elixir` (`0.317x`)
  - `python` (`0.542x`)
  - `typescript` (`0.784x`)

### Main performance gaps

- Runtime slower than `kotlin` (`1.943x`)
- Compile cold slower than:
  - `cpp` (`1.123x`)
  - `c` (`1.112x`)
  - `go` (`1.041x`)

### Concurrency proxy snapshot (`coro-prime-sieve`, lower is better)

- `vibelang`: `1.643 ms`
- `kotlin`: `1.285 ms`
- `go`: `12.341 ms`
- `typescript`: `155.549 ms`
- `python`: `316.813 ms`
- `elixir`: `329.597 ms`

## Why this is still not publishable

- Strict publication mode still fails immediately on parity gating because 4 adapters are still noncanonical:
  - `edigits`
  - `http-server`
  - `json-serde`
  - `secp256k1`
- In this local no-docker run, additional strict gate failures remain for missing language lanes:
  - runtime/compile unavailable: `rust`, `zig`, `swift`
- Docker daemon connectivity was unstable during this session, so strict Docker-backed collection could not be completed end-to-end.

## Action checklist

- Canonical checklist for remaining work: `reports/benchmarks/third_party/analysis/gaps_optimization_blocker_checklist.md`
