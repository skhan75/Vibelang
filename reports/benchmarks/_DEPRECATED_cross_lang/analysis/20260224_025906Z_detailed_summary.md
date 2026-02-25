# Cross-Language Benchmark Detailed Summary (Updated)

- generated_at_utc: `2026-02-24T03:31:23Z`
- source_reports:
  - `reports/benchmarks/cross_lang/full/results.json`
  - `reports/benchmarks/cross_lang/quick/results.json`
  - `reports/benchmarks/cross_lang/latest/summary.md`
- suite: `cross_lang_starter8`
- compared_languages: `vibelang`, `c`, `rust`, `go`, `python`, `typescript`
- host: `AMD Ryzen 9 5900X`, kernel `6.6.87.2-microsoft-standard-WSL2`

## 1) Executive Summary

Full-profile geomean ratios place VibeLang at:

- `2.623x` vs C
- `1.909x` vs Rust
- `2.752x` vs Go
- `0.154x` vs Python
- `0.174x` vs TypeScript

Interpretation:

- Ratio `> 1.0` means VibeLang is slower than that baseline.
- Ratio `< 1.0` means VibeLang is faster than that baseline.

So this is a split result: VibeLang is much faster than Python/TypeScript overall, but still has major hot-path gaps versus C/Rust/Go due to a small number of runtime-heavy cases.

## 2) Method and Stability Signal

- quick profile: warmup=`1`, measured=`5`
- full profile: warmup=`3`, measured=`20`
- quick -> full geomean drift:
  - vs C: `2.683 -> 2.623` (`-2.25%`)
  - vs Rust: `1.942 -> 1.909` (`-1.72%`)
  - vs Go: `2.728 -> 2.752` (`+0.90%`)
  - vs Python: `0.157 -> 0.154` (`-2.13%`)
  - vs TypeScript: `0.182 -> 0.174` (`-4.15%`)

Interpretation: stability is good enough for optimization planning; no directional reversals happened between quick and full.

## 3) Full-Run Per-Case View

| case | Vibe ms | C ms | Rust ms | Go ms | Python ms | TypeScript ms | Vibe/C | Vibe/Rust | Vibe/Go | Vibe/Python | Vibe/TypeScript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fib_recursive | 1.650 | 1.150 | 1.550 | 2.200 | 38.650 | 29.650 | 1.435 | 1.065 | 0.750 | 0.043 | 0.056 |
| prime_sieve | 1.500 | 1.500 | 1.900 | 2.650 | 28.500 | 30.500 | 1.000 | 0.789 | 0.566 | 0.053 | 0.049 |
| sort_int | 2.000 | 1.350 | 1.900 | 2.400 | 62.400 | 44.300 | 1.481 | 1.053 | 0.833 | 0.032 | 0.045 |
| hashmap_frequency | 256.500 | 1.750 | 3.100 | 4.000 | 45.350 | 31.900 | 146.571 | 82.742 | 64.125 | 5.656 | 8.041 |
| string_concat_checksum | 11.100 | 2.900 | 4.000 | 5.700 | 35.900 | 32.300 | 3.828 | 2.775 | 1.947 | 0.309 | 0.344 |
| json_roundtrip | 11.550 | 5.100 | 9.200 | 18.800 | 234.950 | 70.600 | 2.265 | 1.255 | 0.614 | 0.049 | 0.164 |
| channel_pingpong | 2511.850 | 2567.800 | 2546.300 | 13.800 | 3373.350 | 3567.250 | 0.978 | 0.986 | 182.018 | 0.745 | 0.704 |
| worker_pool_reduce | 1.400 | 1.650 | 2.000 | 2.100 | 20.700 | 65.650 | 0.848 | 0.700 | 0.667 | 0.068 | 0.021 |

## 4) Runtime Concentration (Where VibeLang Time Goes)

Using the sum of VibeLang mean wall times:

- `channel_pingpong`: `89.79%`
- `hashmap_frequency`: `9.17%`
- `json_roundtrip`: `0.41%`
- `string_concat_checksum`: `0.40%`
- all remaining five cases combined: `~0.23%`

Interpretation: almost all optimization return comes from channel and map paths.

## 5) Sensitivity Analysis (Outlier Influence)

Geomean ratios under case exclusions:

- baseline (all cases):
  - C `2.623`, Rust `1.909`, Go `2.752`, Python `0.154`, TypeScript `0.174`
- excluding `hashmap_frequency`:
  - C `1.476`, Rust `1.114`, Go `1.755`, Python `0.092`, TypeScript `0.101`
- excluding `channel_pingpong`:
  - C `3.020`, Rust `2.097`, Go `1.512`, Python `0.123`, TypeScript `0.142`
- excluding both `hashmap_frequency` + `channel_pingpong`:
  - C `1.581`, Rust `1.137`, Go `0.810`, Python `0.065`, TypeScript `0.073`

Interpretation:

- `hashmap_frequency` is the dominant factor in VibeLang's C/Rust geomean gap.
- `channel_pingpong` is the dominant factor in the VibeLang vs Go geomean gap.
- Even with those hotspots, VibeLang remains clearly ahead of Python/TypeScript in aggregate.

## 6) Build and Artifact Observations

Average compile elapsed time across 8 cases:

- VibeLang: `353.13 ms`
- C: `95.88 ms`
- Rust: `360.00 ms`
- Go: `114.75 ms`
- Python: `26.00 ms` (syntax check + interpreter execution model)
- TypeScript: `1044.25 ms` (compiler invocation dominates)

Average artifact size (`binary_size_bytes` field):

- VibeLang: `75,388 B` (~`73.6 KB`)
- C: `16,123 B` (~`15.7 KB`)
- Rust: `3,994,623 B` (~`3.81 MB`)
- Go: `1,388,184 B` (~`1.32 MB`)
- Python: `443 B` (script file)
- TypeScript: `680 B` (emitted JS file)

## 7) Practical Bottom Line

VibeLang already behaves like a native language on several compute-oriented workloads and remains far ahead of Python/TypeScript for this suite. The data still points to two priority bottleneck families before it can claim stronger C/Rust/Go parity:

1. map algorithmics and map runtime lowering (`hashmap_frequency`),
2. channel latency fast path (`channel_pingpong`, especially vs Go runtime channels).

Those two areas remain the highest leverage optimization targets for the next phase.
