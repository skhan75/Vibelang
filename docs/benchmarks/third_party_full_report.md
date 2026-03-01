# Third-Party Benchmark Report (PLB-CI, strict)

This is a **user-friendly** summary of the canonical third-party benchmark artifacts:

- `reports/benchmarks/third_party/full/results.json` (machine-readable, canonical)
- `reports/benchmarks/third_party/full/summary.md` (human-readable, canonical)

If you want to reproduce the run, see `benchmarks/third_party/local_full_run_report.md`.

## Run metadata

- **Profile**: `full`
- **generated_at_utc**: `2026-03-01T07:28:22Z`
- **Publication mode**: `strict` (Docker-enabled, pinned PLB-CI ref, no degraded lanes)
- **Pinned PLB-CI ref**: `ad18b203dd1769724f4eea94fc3ac1e99f6593e0`

## What the ratios mean

The tables report the ratio \( \text{VibeLang} / \text{baseline} \).

- **Ratio < 1.0**: VibeLang is faster on average
- **Ratio > 1.0**: VibeLang is slower on average

These are **geomeans over shared problems** for the runtime lane, and **cold-build** ratios for the compile lane.

## Headline results

### Runtime geomean ratios (VibeLang vs baselines)

- **Faster than**: `c` (0.549), `cpp` (0.371), `rust` (0.485), `zig` (0.398), `python` (0.224), `typescript` (0.066), `elixir` (0.028)
- **Slower than**: `go` (13.220), `kotlin` (17.158)

### Compile cold ratios (VibeLang vs baselines)

- **Faster than**: `rust` (0.566), `zig` (0.437)
- **Slower than**: `c` (1.718), `cpp` (1.708), `python` (1.427), `typescript` (1.083), `go` (1.056)

## Lane coverage

Strict validation passed with the following allowlisted gaps:

- **Runtime unavailable**: `swift`
- **Compile unavailable**: `swift`, `kotlin`, `elixir`

All other required lanes in the strict budget policy were present.

## Memory footprint and binary size

This run *does* track memory footprint and (where compile lanes exist) binary sizes:

- **Memory footprint**: averaged `mem_bytes` sampled by the benchmark harness across runs.
- **Binary size**: `binary_size_bytes` from the compile lane output folder (best-effort; what “binary” means varies by language).

Snapshot from `results.json`:

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

## Per-problem highlights (selected)

From `reports/benchmarks/third_party/full/summary.md`:

- **`http-server` (ms)**: vibelang 39.563, go 1.527, kotlin 1.230, python 881.610, rust 236.583  
- **`json-serde` (ms)**: vibelang 3.036, go 1.791, kotlin 1.144, python 59.970, rust 59.445  
- **`coro-prime-sieve` (ms)**: vibelang 3.349, go 2.631, kotlin 1.285, python 153.959, elixir 329.597

## Where to look for full detail

- **Canonical raw data**: `reports/benchmarks/third_party/full/results.json`
- **Canonical summary tables**: `reports/benchmarks/third_party/full/summary.md`
- **Delta report**: `reports/benchmarks/third_party/analysis/deltas/latest_delta.md`

