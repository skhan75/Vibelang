# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-02-25T18:19:23Z`
- budget_status: `warn`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | 0.102 |
| cpp | 0.112 |
| elixir | 0.004 |
| go | 0.037 |
| kotlin | 1.876 |
| python | 0.005 |
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
| elixir | 0.357 |
| go | 1.004 |
| kotlin | 0.318 |
| python | n/a |
| rust | n/a |
| swift | n/a |
| typescript | n/a |
| zig | n/a |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | 4561306 | 1465.615 | 1.643 |
| c | 3729967 | n/a | n/a |
| cpp | 2113536 | n/a | n/a |
| rust | n/a | n/a | n/a |
| go | 9994895 | 1519.072 | 12.490 |
| zig | n/a | n/a | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | 1497.016 | 1.422 |
| elixir | 82242414 | 1442.907 | 314.664 |
| python | 29156181 | n/a | 377.597 |
| typescript | n/a | n/a | n/a |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `0.082682`
- vibelang_incremental_compile_mean_ms: `1465.615`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| binarytrees | 1.531 | n/a | n/a | n/a | 190.955 | n/a | n/a | 1.240 | n/a | 812.815 | n/a |
| coro-prime-sieve | 1.643 | n/a | n/a | n/a | 12.490 | n/a | n/a | 1.422 | 314.664 | 377.597 | n/a |
| edigits | 1.630 | n/a | n/a | n/a | 28.964 | n/a | n/a | n/a | n/a | 394.952 | n/a |
| fannkuch-redux | 1.582 | n/a | 20.406 | n/a | 73.030 | n/a | n/a | n/a | n/a | n/a | n/a |
| fasta | 32.336 | n/a | n/a | n/a | 16.152 | n/a | n/a | n/a | n/a | 463.882 | n/a |
| helloworld | 0.817 | 0.984 | 0.929 | n/a | 1.785 | n/a | n/a | n/a | 254.089 | 43.175 | n/a |
| http-server | 2.285 | n/a | n/a | n/a | 112.957 | n/a | n/a | 1.468 | n/a | 2361.410 | n/a |
| json-serde | 17.306 | n/a | n/a | n/a | 111.390 | n/a | n/a | 1.139 | n/a | 208.751 | n/a |
| knucleotide | 1.823 | 26.671 | n/a | n/a | 82.515 | n/a | n/a | n/a | n/a | 282.846 | n/a |
| lru | 1.464 | n/a | n/a | n/a | 49.953 | n/a | n/a | 1.288 | n/a | 526.075 | n/a |
| mandelbrot | 1.632 | n/a | n/a | n/a | 81.741 | n/a | n/a | n/a | n/a | n/a | n/a |
| merkletrees | 1.638 | n/a | n/a | n/a | 320.512 | n/a | n/a | 1.198 | n/a | n/a | n/a |
| nbody | 2.142 | 30.261 | 18.500 | n/a | 27.610 | n/a | n/a | 1.194 | n/a | 2984.369 | n/a |
| nsieve | 1.636 | 25.210 | 44.726 | n/a | 53.547 | n/a | n/a | n/a | n/a | 823.708 | n/a |
| pidigits | 1.535 | n/a | n/a | n/a | 203.894 | n/a | n/a | 1.230 | 583.510 | 358.349 | n/a |
| regex-redux | 1.726 | n/a | n/a | n/a | 1368.518 | n/a | n/a | 1.230 | n/a | 471.009 | n/a |
| secp256k1 | 4.380 | n/a | n/a | n/a | 20.792 | n/a | n/a | 1.337 | n/a | 733.831 | n/a |
| spectral-norm | 1.397 | 33.463 | 22.798 | n/a | 106.628 | n/a | n/a | n/a | n/a | n/a | n/a |

## Wins

- Runtime: faster than c (ratio=0.102)
- Runtime: faster than cpp (ratio=0.112)
- Runtime: faster than go (ratio=0.037)
- Runtime: faster than elixir (ratio=0.004)
- Runtime: faster than python (ratio=0.005)
- Compile: faster than kotlin (ratio=0.318)
- Compile: faster than elixir (ratio=0.357)

## Gaps and Improvement Opportunities

- Runtime: slower than kotlin (ratio=1.876)
- Compile: slower than go (ratio=1.004)

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

