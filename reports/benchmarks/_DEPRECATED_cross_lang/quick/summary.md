# Cross-Language Benchmark Summary

- profile: `quick`
- generated_at_utc: `2026-02-24T09:59:48Z`
- runs: warmup=1 measured=5
- cpu_model: `AMD Ryzen 9 5900X 12-Core Processor`
- kernel_release: `6.6.87.2-microsoft-standard-WSL2`
- cpu_governor: `unavailable`
- is_wsl: `True`

| case | vibelang mean ms | c mean ms | rust mean ms | go mean ms | python mean ms | typescript mean ms | vibe/c | vibe/rust | vibe/go | vibe/python | vibe/typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fib_recursive | 2.000 | 2.000 | 2.000 | 2.200 | 40.200 | 35.800 | 1.000 | 1.000 | 0.909 | 0.050 | 0.056 |
| prime_sieve | 2.200 | 1.800 | 2.400 | 3.400 | 32.000 | 36.000 | 1.222 | 0.917 | 0.647 | 0.069 | 0.061 |
| sort_int | 2.200 | 1.000 | 2.000 | 2.400 | 59.600 | 42.800 | 2.200 | 1.100 | 0.917 | 0.037 | 0.051 |
| hashmap_frequency | 18.400 | 1.600 | 3.000 | 6.000 | 46.200 | 31.200 | 11.500 | 6.133 | 3.067 | 0.398 | 0.590 |
| hashmap_frequency_int_key | 5.400 | 1.600 | 3.000 | 4.000 | 45.400 | 31.200 | 3.375 | 1.800 | 1.350 | 0.119 | 0.173 |
| hashmap_frequency_str_key | 17.000 | 20.000 | 10.600 | 8.200 | 69.600 | 33.600 | 0.850 | 1.604 | 2.073 | 0.244 | 0.506 |
| string_concat_checksum | 7.200 | 3.000 | 4.000 | 6.000 | 35.400 | 33.600 | 2.400 | 1.800 | 1.200 | 0.203 | 0.214 |
| json_roundtrip | 9.800 | 6.600 | 8.600 | 17.600 | 230.000 | 70.600 | 1.485 | 1.140 | 0.557 | 0.043 | 0.139 |
| channel_pingpong | 2103.600 | 1855.000 | 1734.400 | 12.800 | 2444.600 | 2717.600 | 1.134 | 1.213 | 164.344 | 0.861 | 0.774 |
| worker_pool_reduce | 1.200 | 1.600 | 2.000 | 2.200 | 20.600 | 64.800 | 0.750 | 0.600 | 0.545 | 0.058 | 0.019 |

## Geomean Ratios

- vibelang_vs_c: `1.750`
- vibelang_vs_rust: `1.389`
- vibelang_vs_go: `1.755`
- vibelang_vs_python: `0.120`
- vibelang_vs_typescript: `0.144`

Interpretation: ratio > 1.0 means VibeLang mean runtime is slower than the baseline; ratio < 1.0 means faster.

## Fairness Notes

- Native AOT languages (VibeLang/C/Rust/Go) are compared in the same suite, while Python/TypeScript are interpreter/JIT-oriented baselines.
- Channel and scheduler semantics differ by runtime implementation; cross-language ratios in concurrency cases should be interpreted with this caveat.
- Host context matters: WSL2 and native Linux can differ on scheduler and timing behavior.

## Delta vs Baseline

- baseline: `reports/benchmarks/cross_lang/full/results.json`
- baseline_generated_at_utc: `2026-02-24T09:53:49Z`

### Geomean Delta (Vibe/Baseline Ratios)

| baseline | before | after | delta_abs | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| c | 1.831 | 1.750 | -0.082 | -4.46% |
| rust | 1.370 | 1.389 | 0.019 | 1.37% |
| go | 1.826 | 1.755 | -0.071 | -3.88% |
| python | 0.118 | 0.120 | 0.002 | 1.68% |
| typescript | 0.138 | 0.144 | 0.006 | 4.31% |

### Per-Case Vibe Runtime Delta

| case | vibe_before_ms | vibe_after_ms | delta_abs_ms | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| fib_recursive | 1.900 | 2.000 | 0.100 | 5.26% |
| prime_sieve | 1.800 | 2.200 | 0.400 | 22.22% |
| sort_int | 2.000 | 2.200 | 0.200 | 10.00% |
| hashmap_frequency | 17.550 | 18.400 | 0.850 | 4.84% |
| hashmap_frequency_int_key | 5.200 | 5.400 | 0.200 | 3.85% |
| hashmap_frequency_str_key | 17.950 | 17.000 | -0.950 | -5.29% |
| string_concat_checksum | 7.050 | 7.200 | 0.150 | 2.13% |
| json_roundtrip | 9.450 | 9.800 | 0.350 | 3.70% |
| channel_pingpong | 2462.000 | 2103.600 | -358.400 | -14.56% |
| worker_pool_reduce | 1.250 | 1.200 | -0.050 | -4.00% |

## Budget Gate

- status: `pass`
- no budget issues detected

