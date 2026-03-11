# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-03-11T08:09:15Z`
- budget_status: `warn`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | 8.324 |
| cpp | 12.510 |
| elixir | 0.027 |
| go | 56.132 |
| kotlin | 40.618 |
| python | 0.583 |
| rust | 1.627 |
| swift | n/a |
| typescript | 0.211 |
| zig | 2.194 |

Interpretation: ratio > 1.0 means VibeLang is slower on average; ratio < 1.0 means faster.

## Compile Cold Ratios (VibeLang vs Baselines)

| baseline | vibelang_cold_ratio |
| --- | ---: |
| c | 1.869 |
| cpp | 1.922 |
| elixir | n/a |
| go | 1.140 |
| kotlin | n/a |
| python | 1.595 |
| rust | 0.654 |
| swift | n/a |
| typescript | 1.194 |
| zig | 0.481 |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | 441542295 | 1512.008 | 3.342 |
| c | 3664128 | 1512.326 | n/a |
| cpp | 2015232 | 1537.320 | n/a |
| rust | 8045008 | 1522.698 | 77.243 |
| go | 1966080 | 1496.747 | 2.623 |
| zig | 3731911 | 1522.224 | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | n/a | 1.285 |
| elixir | 83461266 | n/a | 329.597 |
| python | 27197952 | 1523.742 | 143.837 |
| typescript | 78095616 | 1547.105 | 129.614 |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `0.034904`
- vibelang_incremental_compile_mean_ms: `1512.008`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| binarytrees | 14.451 | n/a | n/a | 94.441 | 1.306 | 113.886 | n/a | 1.587 | n/a | 182.275 | 116.683 |
| coro-prime-sieve | 3.342 | n/a | n/a | 77.243 | 2.623 | n/a | n/a | 1.285 | 329.597 | 143.837 | 129.614 |
| edigits | 878.303 | n/a | n/a | 39.063 | 1.216 | 280.432 | n/a | n/a | n/a | 28.420 | n/a |
| fannkuch-redux | 604.145 | n/a | 18.490 | 64.041 | 1.174 | 97.870 | n/a | n/a | n/a | n/a | n/a |
| fasta | 189.547 | n/a | n/a | 13.563 | 1.183 | 8.052 | n/a | n/a | n/a | 115.869 | 194.047 |
| helloworld | 1.668 | 1.119 | 1.176 | 1.065 | 1.899 | 0.927 | n/a | n/a | 269.005 | 27.318 | 32.293 |
| http-server | 39.624 | n/a | n/a | 258.582 | 1.331 | n/a | n/a | 1.230 | n/a | 880.331 | 289.704 |
| json-serde | 2.864 | n/a | n/a | 54.343 | 1.384 | n/a | n/a | 1.144 | n/a | 56.135 | 151.651 |
| knucleotide | n/a | 28.660 | n/a | 39.699 | 1.274 | 70.322 | n/a | n/a | n/a | 67.734 | n/a |
| lru | 12.300 | n/a | n/a | 19.027 | 1.351 | 11.379 | n/a | 1.132 | n/a | 122.630 | 126.350 |
| mandelbrot | n/a | n/a | n/a | 10.554 | 1.156 | 9.210 | n/a | n/a | n/a | n/a | n/a |
| merkletrees | 14.063 | n/a | n/a | 115.454 | 1.272 | 138.676 | n/a | 1.202 | n/a | 1363.562 | 140.476 |
| nbody | 1078.831 | 23.427 | 18.319 | 18.776 | 1.235 | 20.689 | n/a | 1.345 | n/a | 595.836 | 68.614 |
| nsieve | 355.824 | 35.990 | 50.819 | 60.622 | 2.798 | 59.074 | n/a | n/a | n/a | 355.684 | n/a |
| pidigits | 202.179 | n/a | n/a | 240.331 | 1.174 | 405.921 | n/a | 1.101 | 627.014 | 121.003 | 989.151 |
| regex-redux | 4724.021 | n/a | n/a | 43.220 | 1.362 | n/a | n/a | 1.209 | n/a | 127.309 | n/a |
| secp256k1 | 96.932 | n/a | n/a | 141.412 | 1.278 | n/a | n/a | 1.107 | n/a | 211.970 | 386.525 |
| spectral-norm | 531.234 | 75.121 | 33.125 | 91.899 | 1.260 | 166.487 | n/a | n/a | n/a | 1.220 | 219.733 |

## Wins

- Runtime: faster than elixir (ratio=0.027)
- Runtime: faster than python (ratio=0.583)
- Runtime: faster than typescript (ratio=0.211)
- Compile: faster than rust (ratio=0.654)
- Compile: faster than zig (ratio=0.481)

## Gaps and Improvement Opportunities

- Runtime: slower than go (ratio=56.132)
- Runtime: slower than kotlin (ratio=40.618)
- Runtime: slower than cpp (ratio=12.510)
- Runtime: slower than c (ratio=8.324)
- Runtime: slower than zig (ratio=2.194)
- Compile: slower than cpp (ratio=1.922)

## Simple-language analysis

- VibeLang still has performance gaps versus some baselines. Focus next on the worst ratios first.
- There are measurable strengths that can be highlighted in public benchmark notes.
- Keep fairness caveats explicit: toolchain versions, host environment, and benchmark semantics affect results.

## Budget Gate Output

- mode: `warn`
- status: `warn`
- warnings:
  - runtime ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `kotlin`
  - compile ratio missing/zero for baseline `elixir`
  - runtime language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `kotlin` unavailable but allowlisted (status=unavailable)
  - compile language `elixir` unavailable but allowlisted (status=unavailable)
  - [warn-mode] runtime geomean ratio exceeded for c: current=8.324 limit=6.000
  - [warn-mode] runtime geomean ratio exceeded for cpp: current=12.510 limit=5.000
  - [warn-mode] runtime geomean ratio exceeded for go: current=56.132 limit=20.000
  - [warn-mode] runtime geomean ratio exceeded for kotlin: current=40.618 limit=25.000
  - [warn-mode] runtime geomean ratio exceeded for python: current=0.583 limit=0.450
  - [warn-mode] compile cold ratio exceeded for python: current=1.595 limit=1.500

