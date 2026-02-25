# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-02-25T09:10:20Z`
- budget_status: `warn`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | 0.794 |
| cpp | 0.683 |
| elixir | 0.003 |
| go | 0.426 |
| kotlin | n/a |
| python | 0.022 |
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
| elixir | 0.313 |
| go | 1.006 |
| kotlin | 0.305 |
| python | n/a |
| rust | n/a |
| swift | n/a |
| typescript | n/a |
| zig | n/a |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | 1179648 | 1531.961 | n/a |
| c | 1376256 | n/a | n/a |
| cpp | 1376256 | n/a | n/a |
| rust | n/a | n/a | n/a |
| go | 1966080 | 1500.216 | 3.843 |
| zig | n/a | n/a | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | 1591.665 | 1.469 |
| elixir | 81420288 | 1449.688 | 257.215 |
| python | 15669248 | n/a | 178.397 |
| typescript | n/a | n/a | n/a |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `0.070354`
- vibelang_incremental_compile_mean_ms: `1531.961`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| coro-prime-sieve | n/a | n/a | n/a | n/a | 3.843 | n/a | n/a | 1.469 | 257.215 | 178.397 | n/a |
| helloworld | 0.869 | 0.999 | 0.997 | n/a | 1.899 | n/a | n/a | n/a | 272.769 | 44.088 | n/a |
| nsieve | 1.700 | 2.346 | 3.172 | n/a | 4.296 | n/a | n/a | n/a | n/a | 67.289 | n/a |

## Wins

- Runtime: faster than c (ratio=0.794)
- Runtime: faster than cpp (ratio=0.683)
- Runtime: faster than go (ratio=0.426)
- Runtime: faster than elixir (ratio=0.003)
- Runtime: faster than python (ratio=0.022)
- Compile: faster than kotlin (ratio=0.305)
- Compile: faster than elixir (ratio=0.313)

## Gaps and Improvement Opportunities

- Compile: slower than go (ratio=1.006)

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
  - runtime ratio missing/zero for baseline `kotlin`
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

