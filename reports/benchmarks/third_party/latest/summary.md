# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-02-25T18:01:55Z`
- budget_status: `warn`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | 0.090 |
| cpp | 0.100 |
| elixir | 0.003 |
| go | 0.047 |
| kotlin | 2.696 |
| python | 0.006 |
| rust | n/a |
| swift | n/a |
| typescript | n/a |
| zig | n/a |

Interpretation: ratio > 1.0 means VibeLang is slower on average; ratio < 1.0 means faster.

## Compile Cold Ratios (VibeLang vs Baselines)

| baseline | vibelang_cold_ratio |
| --- | ---: |
| c | n/a |
| cpp | n/a |
| elixir | 0.355 |
| go | 1.043 |
| kotlin | 0.312 |
| python | n/a |
| rust | n/a |
| swift | n/a |
| typescript | n/a |
| zig | n/a |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | 4561306 | 1475.265 | 1.502 |
| c | 3747840 | n/a | n/a |
| cpp | 2088960 | n/a | n/a |
| rust | n/a | n/a | n/a |
| go | 10165125 | 1461.835 | 12.794 |
| zig | n/a | n/a | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | 1458.916 | 1.274 |
| elixir | 83651438 | 1522.440 | 316.059 |
| python | 29143893 | n/a | 375.974 |
| typescript | n/a | n/a | n/a |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `0.067022`
- vibelang_incremental_compile_mean_ms: `1475.265`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| binarytrees | n/a | n/a | n/a | n/a | 197.622 | n/a | n/a | 1.128 | n/a | 744.006 | n/a |
| coro-prime-sieve | 1.502 | n/a | n/a | n/a | 12.794 | n/a | n/a | 1.274 | 316.059 | 375.974 | n/a |
| edigits | 1.798 | n/a | n/a | n/a | 28.514 | n/a | n/a | n/a | n/a | 443.621 | n/a |
| fannkuch-redux | 1.500 | n/a | 20.299 | n/a | 73.280 | n/a | n/a | n/a | n/a | n/a | n/a |
| fasta | 31.747 | n/a | n/a | n/a | 16.377 | n/a | n/a | n/a | n/a | 511.697 | n/a |
| helloworld | 0.820 | 0.975 | 0.915 | n/a | 1.798 | n/a | n/a | n/a | 282.833 | 43.511 | n/a |
| http-server | 2.326 | n/a | n/a | n/a | 110.612 | n/a | n/a | 0.998 | n/a | 2859.644 | n/a |
| json-serde | 17.445 | n/a | n/a | n/a | 111.117 | n/a | n/a | 0.926 | n/a | 203.225 | n/a |
| knucleotide | 1.692 | 27.412 | n/a | n/a | 82.508 | n/a | n/a | n/a | n/a | 277.648 | n/a |
| lru | n/a | n/a | n/a | n/a | 51.262 | n/a | n/a | 0.948 | n/a | 535.251 | n/a |
| mandelbrot | 1.610 | n/a | n/a | n/a | 80.143 | n/a | n/a | n/a | n/a | n/a | n/a |
| merkletrees | n/a | n/a | n/a | n/a | 317.394 | n/a | n/a | 1.116 | n/a | n/a | n/a |
| nbody | 1.775 | 30.875 | 18.407 | n/a | 28.793 | n/a | n/a | 1.000 | n/a | 3017.021 | n/a |
| nsieve | 1.610 | 27.373 | 43.645 | n/a | 51.729 | n/a | n/a | n/a | n/a | 861.083 | n/a |
| pidigits | 1.507 | n/a | n/a | n/a | 195.734 | n/a | n/a | 1.057 | 657.710 | 358.068 | n/a |
| regex-redux | 1.857 | n/a | n/a | n/a | 1280.790 | n/a | n/a | 1.051 | n/a | 476.350 | n/a |
| secp256k1 | 4.720 | n/a | n/a | n/a | 19.141 | n/a | n/a | 1.053 | n/a | 738.057 | n/a |
| spectral-norm | 1.371 | 41.728 | 32.214 | n/a | 102.324 | n/a | n/a | n/a | n/a | n/a | n/a |

## Wins

- Runtime: faster than c (ratio=0.090)
- Runtime: faster than cpp (ratio=0.100)
- Runtime: faster than go (ratio=0.047)
- Runtime: faster than elixir (ratio=0.003)
- Runtime: faster than python (ratio=0.006)
- Compile: faster than kotlin (ratio=0.312)
- Compile: faster than elixir (ratio=0.355)

## Gaps and Improvement Opportunities

- Runtime: slower than kotlin (ratio=2.696)
- Compile: slower than go (ratio=1.043)

## Simple-language analysis

- VibeLang still has performance gaps versus some baselines. Focus next on the worst ratios first.
- There are measurable strengths that can be highlighted in public benchmark notes.
- Keep fairness caveats explicit: toolchain versions, host environment, and benchmark semantics affect results.

## Budget Gate Output

- mode: `warn`
- status: `warn`
- warnings:
  - runtime ratio missing/zero for baseline `rust`
  - runtime ratio missing/zero for baseline `zig`
  - runtime ratio missing/zero for baseline `swift`
  - runtime ratio missing/zero for baseline `typescript`
  - compile ratio missing/zero for baseline `c`
  - compile ratio missing/zero for baseline `cpp`
  - compile ratio missing/zero for baseline `rust`
  - compile ratio missing/zero for baseline `zig`
  - compile ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `python`
  - compile ratio missing/zero for baseline `typescript`
  - runtime language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `swift` unavailable but allowlisted (status=unavailable)
  - [warn-mode] required runtime language `rust` not available (status=unavailable)
  - [warn-mode] required runtime language `zig` not available (status=unavailable)
  - [warn-mode] required runtime language `typescript` not available (status=unavailable)
  - [warn-mode] required compile language `c` not available (status=unavailable)
  - [warn-mode] required compile language `cpp` not available (status=unavailable)
  - [warn-mode] required compile language `rust` not available (status=unavailable)
  - [warn-mode] required compile language `zig` not available (status=unavailable)
  - [warn-mode] required compile language `python` not available (status=unavailable)
  - [warn-mode] required compile language `typescript` not available (status=unavailable)

