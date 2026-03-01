# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-03-01T05:34:10Z`
- budget_status: `fail`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | 0.650 |
| cpp | 0.425 |
| elixir | 0.029 |
| go | 15.159 |
| kotlin | 17.484 |
| python | 0.230 |
| rust | 0.501 |
| swift | n/a |
| typescript | 0.067 |
| zig | 0.629 |

Interpretation: ratio > 1.0 means VibeLang is slower on average; ratio < 1.0 means faster.

## Compile Cold Ratios (VibeLang vs Baselines)

| baseline | vibelang_cold_ratio |
| --- | ---: |
| c | n/a |
| cpp | n/a |
| elixir | n/a |
| go | 1.069 |
| kotlin | n/a |
| python | n/a |
| rust | 0.567 |
| swift | n/a |
| typescript | 1.047 |
| zig | n/a |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | 33148928 | 1481.320 | 3.785 |
| c | 3619840 | n/a | n/a |
| cpp | 1998848 | n/a | n/a |
| rust | 8033956 | 1501.316 | 78.892 |
| go | 1966080 | 1493.640 | 2.497 |
| zig | 196608 | n/a | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | n/a | 1.285 |
| elixir | 83461266 | n/a | 329.597 |
| python | 27247957 | n/a | 147.569 |
| typescript | 77927680 | 1510.422 | 133.180 |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `0.072217`
- vibelang_incremental_compile_mean_ms: `1481.320`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| binarytrees | 2.942 | n/a | n/a | 90.915 | 1.229 | n/a | n/a | 1.587 | n/a | 189.246 | 127.319 |
| coro-prime-sieve | 3.785 | n/a | n/a | 78.892 | 2.497 | n/a | n/a | 1.285 | 329.597 | 147.569 | 133.180 |
| edigits | 942.642 | n/a | n/a | 40.512 | 1.170 | n/a | n/a | n/a | n/a | 28.523 | n/a |
| fannkuch-redux | 3.093 | n/a | 20.221 | 66.667 | 1.122 | 60.334 | n/a | n/a | n/a | n/a | n/a |
| fasta | 195.784 | n/a | n/a | 14.069 | 1.255 | 7.524 | n/a | n/a | n/a | 125.147 | 198.993 |
| helloworld | 1.732 | 0.988 | 0.937 | 1.185 | 1.899 | n/a | n/a | n/a | 269.005 | 28.293 | 32.828 |
| http-server | 38.369 | n/a | n/a | 164.807 | 1.120 | n/a | n/a | 1.230 | n/a | 885.209 | 276.192 |
| json-serde | 3.017 | n/a | n/a | 56.572 | 1.125 | n/a | n/a | 1.144 | n/a | 64.433 | 149.052 |
| knucleotide | n/a | 25.320 | n/a | 38.356 | 1.248 | n/a | n/a | n/a | n/a | 75.249 | n/a |
| lru | 14.254 | n/a | n/a | 19.246 | 3.688 | n/a | n/a | 1.132 | n/a | 128.110 | 132.431 |
| mandelbrot | 4.469 | n/a | n/a | 11.272 | 1.466 | n/a | n/a | n/a | n/a | n/a | n/a |
| merkletrees | 3.209 | n/a | n/a | 108.251 | 1.514 | n/a | n/a | 1.202 | n/a | 1378.709 | 142.642 |
| nbody | 3.761 | 27.703 | 18.046 | 19.207 | 1.459 | 20.149 | n/a | 1.345 | n/a | 625.986 | 68.655 |
| nsieve | 380.327 | 35.607 | 50.721 | 60.003 | 2.827 | n/a | n/a | n/a | n/a | 374.218 | n/a |
| pidigits | 204.544 | n/a | n/a | 239.321 | 1.222 | n/a | n/a | 1.101 | 627.014 | 120.161 | 1018.126 |
| regex-redux | 4803.055 | n/a | n/a | 45.872 | 1.223 | n/a | n/a | 1.209 | n/a | 133.715 | n/a |
| secp256k1 | 94.866 | n/a | n/a | 144.830 | 1.168 | n/a | n/a | 1.107 | n/a | 220.471 | 399.762 |
| spectral-norm | 3.017 | 43.001 | 96.088 | 104.062 | 1.248 | n/a | n/a | n/a | n/a | 1.220 | 223.859 |

## Wins

- Runtime: faster than c (ratio=0.650)
- Runtime: faster than cpp (ratio=0.425)
- Runtime: faster than rust (ratio=0.501)
- Runtime: faster than zig (ratio=0.629)
- Runtime: faster than elixir (ratio=0.029)
- Runtime: faster than python (ratio=0.230)
- Runtime: faster than typescript (ratio=0.067)
- Compile: faster than rust (ratio=0.567)

## Gaps and Improvement Opportunities

- Runtime: slower than kotlin (ratio=17.484)
- Runtime: slower than go (ratio=15.159)
- Compile: slower than go (ratio=1.069)
- Compile: slower than typescript (ratio=1.047)

## Simple-language analysis

- VibeLang still has performance gaps versus some baselines. Focus next on the worst ratios first.
- There are measurable strengths that can be highlighted in public benchmark notes.
- Keep fairness caveats explicit: toolchain versions, host environment, and benchmark semantics affect results.

## Budget Gate Output

- mode: `strict`
- status: `fail`
- violations:
  - required compile language `c` not available (status=unavailable)
  - required compile language `cpp` not available (status=unavailable)
  - required compile language `zig` not available (status=unavailable)
  - required compile language `python` not available (status=unavailable)
- warnings:
  - runtime ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `c`
  - compile ratio missing/zero for baseline `cpp`
  - compile ratio missing/zero for baseline `zig`
  - compile ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `kotlin`
  - compile ratio missing/zero for baseline `elixir`
  - compile ratio missing/zero for baseline `python`
  - runtime language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `kotlin` unavailable but allowlisted (status=unavailable)
  - compile language `elixir` unavailable but allowlisted (status=unavailable)

