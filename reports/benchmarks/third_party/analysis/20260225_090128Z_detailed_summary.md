# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-02-25T09:01:28Z`
- budget_status: `warn`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | n/a |
| cpp | n/a |
| elixir | n/a |
| go | n/a |
| kotlin | n/a |
| python | n/a |
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
| elixir | n/a |
| go | n/a |
| kotlin | n/a |
| python | n/a |
| rust | n/a |
| swift | n/a |
| typescript | n/a |
| zig | n/a |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | n/a | n/a | n/a |
| c | 1376256 | n/a | n/a |
| cpp | 1376256 | n/a | n/a |
| rust | n/a | n/a | n/a |
| go | 1966080 | 1536.415 | 3.684 |
| zig | n/a | n/a | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | 1492.425 | 1.252 |
| elixir | 80416768 | 1476.822 | 262.442 |
| python | 15597568 | n/a | 173.072 |
| typescript | n/a | n/a | n/a |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `n/a`
- vibelang_incremental_compile_mean_ms: `n/a`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| coro-prime-sieve | n/a | n/a | n/a | n/a | 3.684 | n/a | n/a | 1.252 | 262.442 | 173.072 | n/a |
| helloworld | n/a | 1.101 | 0.901 | n/a | 1.866 | n/a | n/a | n/a | 269.964 | 46.182 | n/a |
| nsieve | n/a | 2.406 | 3.219 | n/a | 4.085 | n/a | n/a | n/a | n/a | 68.144 | n/a |

## Wins

- No clear wins in this run.

## Gaps and Improvement Opportunities

- No major gaps detected.

## Simple-language analysis

- Current run shows stable competitiveness against configured baselines.
- Keep fairness caveats explicit: toolchain versions, host environment, and benchmark semantics affect results.

## Budget Gate Output

- mode: `warn`
- status: `warn`
- warnings:
  - runtime ratio missing/zero for baseline `c`
  - runtime ratio missing/zero for baseline `cpp`
  - runtime ratio missing/zero for baseline `rust`
  - runtime ratio missing/zero for baseline `go`
  - runtime ratio missing/zero for baseline `zig`
  - runtime ratio missing/zero for baseline `swift`
  - runtime ratio missing/zero for baseline `kotlin`
  - runtime ratio missing/zero for baseline `elixir`
  - runtime ratio missing/zero for baseline `python`
  - runtime ratio missing/zero for baseline `typescript`
  - compile ratio missing/zero for baseline `c`
  - compile ratio missing/zero for baseline `cpp`
  - compile ratio missing/zero for baseline `rust`
  - compile ratio missing/zero for baseline `go`
  - compile ratio missing/zero for baseline `zig`
  - compile ratio missing/zero for baseline `swift`
  - compile ratio missing/zero for baseline `kotlin`
  - compile ratio missing/zero for baseline `elixir`
  - compile ratio missing/zero for baseline `python`
  - compile ratio missing/zero for baseline `typescript`
  - runtime language `swift` unavailable but allowlisted (status=unavailable)
  - compile language `swift` unavailable but allowlisted (status=unavailable)
  - [warn-mode] required runtime language `vibelang` not available (status=unavailable)
  - [warn-mode] required runtime language `rust` not available (status=unavailable)
  - [warn-mode] required runtime language `zig` not available (status=unavailable)
  - [warn-mode] required runtime language `typescript` not available (status=unavailable)
  - [warn-mode] required compile language `vibelang` not available (status=unavailable)
  - [warn-mode] required compile language `c` not available (status=unavailable)
  - [warn-mode] required compile language `cpp` not available (status=unavailable)
  - [warn-mode] required compile language `rust` not available (status=unavailable)
  - [warn-mode] required compile language `zig` not available (status=unavailable)
  - [warn-mode] required compile language `python` not available (status=unavailable)
  - [warn-mode] required compile language `typescript` not available (status=unavailable)

