# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-03-01T07:28:22Z`
- budget_status: `pass`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | 0.549 |
| cpp | 0.371 |
| elixir | 0.028 |
| go | 13.220 |
| kotlin | 17.158 |
| python | 0.224 |
| rust | 0.485 |
| swift | n/a |
| typescript | 0.066 |
| zig | 0.398 |

Interpretation: ratio > 1.0 means VibeLang is slower on average; ratio < 1.0 means faster.

## Compile Cold Ratios (VibeLang vs Baselines)

| baseline | vibelang_cold_ratio |
| --- | ---: |
| c | 1.718 |
| cpp | 1.708 |
| elixir | n/a |
| go | 1.056 |
| kotlin | n/a |
| python | 1.427 |
| rust | 0.566 |
| swift | n/a |
| typescript | 1.083 |
| zig | 0.437 |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | 33273670 | 1532.894 | 3.349 |
| c | 3670528 | 1572.853 | n/a |
| cpp | 2031616 | 1547.859 | n/a |
| rust | 8055286 | 1551.427 | 82.803 |
| go | 1966080 | 1559.399 | 2.631 |
| zig | 3732594 | 1555.644 | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | n/a | 1.285 |
| elixir | 83461266 | n/a | 329.597 |
| python | 27168427 | 1547.276 | 153.959 |
| typescript | 78215680 | 1528.079 | 133.574 |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `0.053558`
- vibelang_incremental_compile_mean_ms: `1532.894`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| binarytrees | 2.983 | n/a | n/a | 92.438 | 1.504 | 110.916 | n/a | 1.587 | n/a | 200.065 | 119.021 |
| coro-prime-sieve | 3.349 | n/a | n/a | 82.803 | 2.631 | n/a | n/a | 1.285 | 329.597 | 153.959 | 133.574 |
| edigits | 954.110 | n/a | n/a | 40.493 | 1.502 | 280.855 | n/a | n/a | n/a | 32.538 | n/a |
| fannkuch-redux | 3.675 | n/a | 24.254 | 66.302 | 1.605 | 97.812 | n/a | n/a | n/a | n/a | n/a |
| fasta | 195.761 | n/a | n/a | 14.036 | 1.361 | 7.897 | n/a | n/a | n/a | 130.656 | 207.396 |
| helloworld | 1.681 | 1.076 | 1.223 | 1.084 | 1.899 | 0.937 | n/a | n/a | 269.005 | 28.002 | 32.998 |
| http-server | 39.563 | n/a | n/a | 236.583 | 1.527 | n/a | n/a | 1.230 | n/a | 881.610 | 265.491 |
| json-serde | 3.036 | n/a | n/a | 59.445 | 1.791 | n/a | n/a | 1.144 | n/a | 59.970 | 149.842 |
| knucleotide | n/a | 30.229 | n/a | 46.591 | 1.429 | 68.750 | n/a | n/a | n/a | 72.200 | n/a |
| lru | 11.242 | n/a | n/a | 20.129 | 1.528 | 11.975 | n/a | 1.132 | n/a | 133.439 | 132.231 |
| mandelbrot | 3.388 | n/a | n/a | 11.037 | 1.712 | 9.428 | n/a | n/a | n/a | n/a | n/a |
| merkletrees | 3.550 | n/a | n/a | 112.991 | 1.659 | 128.399 | n/a | 1.202 | n/a | 1438.941 | 148.199 |
| nbody | 3.631 | 23.508 | 20.141 | 19.270 | 1.492 | 20.904 | n/a | 1.345 | n/a | 624.875 | 73.632 |
| nsieve | 369.001 | 35.313 | 55.173 | 59.269 | 2.977 | 58.300 | n/a | n/a | n/a | 362.863 | n/a |
| pidigits | 216.570 | n/a | n/a | 246.322 | 1.478 | 419.525 | n/a | 1.101 | 627.014 | 122.743 | 1042.491 |
| regex-redux | 4769.077 | n/a | n/a | 45.032 | 1.665 | n/a | n/a | 1.209 | n/a | 134.776 | n/a |
| secp256k1 | 95.322 | n/a | n/a | 141.816 | 1.393 | n/a | n/a | 1.107 | n/a | 222.328 | 399.820 |
| spectral-norm | 3.350 | 92.668 | 119.571 | 90.670 | 1.615 | 170.547 | n/a | n/a | n/a | 1.220 | 224.146 |

## Wins

- Runtime: faster than c (ratio=0.549)
- Runtime: faster than cpp (ratio=0.371)
- Runtime: faster than rust (ratio=0.485)
- Runtime: faster than zig (ratio=0.398)
- Runtime: faster than elixir (ratio=0.028)
- Runtime: faster than python (ratio=0.224)
- Runtime: faster than typescript (ratio=0.066)
- Compile: faster than rust (ratio=0.566)
- Compile: faster than zig (ratio=0.437)

## Gaps and Improvement Opportunities

- Runtime: slower than kotlin (ratio=17.158)
- Runtime: slower than go (ratio=13.220)
- Compile: slower than c (ratio=1.718)
- Compile: slower than cpp (ratio=1.708)
- Compile: slower than python (ratio=1.427)
- Compile: slower than typescript (ratio=1.083)

## Simple-language analysis

- VibeLang still has performance gaps versus some baselines. Focus next on the worst ratios first.
- There are measurable strengths that can be highlighted in public benchmark notes.
- Keep fairness caveats explicit: toolchain versions, host environment, and benchmark semantics affect results.

## Budget Gate Output

- mode: `strict`
- status: `pass`
- warnings:
  - runtime ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `kotlin`
  - compile ratio missing/zero for baseline `elixir`
  - runtime language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `kotlin` unavailable but allowlisted (status=unavailable)
  - compile language `elixir` unavailable but allowlisted (status=unavailable)

