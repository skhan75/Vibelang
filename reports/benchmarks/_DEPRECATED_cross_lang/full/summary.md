# Cross-Language Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-02-24T09:53:49Z`
- runs: warmup=3 measured=20
- cpu_model: `AMD Ryzen 9 5900X 12-Core Processor`
- kernel_release: `6.6.87.2-microsoft-standard-WSL2`
- cpu_governor: `unavailable`
- is_wsl: `True`

| case | vibelang mean ms | c mean ms | rust mean ms | go mean ms | python mean ms | typescript mean ms | vibe/c | vibe/rust | vibe/go | vibe/python | vibe/typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fib_recursive | 1.900 | 1.250 | 1.900 | 2.400 | 39.100 | 30.700 | 1.520 | 1.000 | 0.792 | 0.049 | 0.062 |
| prime_sieve | 1.800 | 2.000 | 1.950 | 2.800 | 28.400 | 33.550 | 0.900 | 0.923 | 0.643 | 0.063 | 0.054 |
| sort_int | 2.000 | 1.250 | 1.600 | 2.100 | 58.800 | 45.200 | 1.600 | 1.250 | 0.952 | 0.034 | 0.044 |
| hashmap_frequency | 17.550 | 1.550 | 3.050 | 4.000 | 46.600 | 32.550 | 11.323 | 5.754 | 4.388 | 0.377 | 0.539 |
| hashmap_frequency_int_key | 5.200 | 1.500 | 3.050 | 4.000 | 46.500 | 32.100 | 3.467 | 1.705 | 1.300 | 0.112 | 0.162 |
| hashmap_frequency_str_key | 17.950 | 17.500 | 9.550 | 8.950 | 68.700 | 37.700 | 1.026 | 1.880 | 2.006 | 0.261 | 0.476 |
| string_concat_checksum | 7.050 | 2.700 | 4.050 | 5.350 | 35.000 | 33.900 | 2.611 | 1.741 | 1.318 | 0.201 | 0.208 |
| json_roundtrip | 9.450 | 5.050 | 9.050 | 19.050 | 232.200 | 72.450 | 1.871 | 1.044 | 0.496 | 0.041 | 0.130 |
| channel_pingpong | 2462.000 | 2498.650 | 2558.300 | 13.550 | 2767.750 | 2952.450 | 0.985 | 0.962 | 181.697 | 0.890 | 0.834 |
| worker_pool_reduce | 1.250 | 1.250 | 2.000 | 2.000 | 19.550 | 68.150 | 1.000 | 0.625 | 0.625 | 0.064 | 0.018 |

## Geomean Ratios

- vibelang_vs_c: `1.831`
- vibelang_vs_rust: `1.370`
- vibelang_vs_go: `1.826`
- vibelang_vs_python: `0.118`
- vibelang_vs_typescript: `0.138`

Interpretation: ratio > 1.0 means VibeLang mean runtime is slower than the baseline; ratio < 1.0 means faster.

## Fairness Notes

- Native AOT languages (VibeLang/C/Rust/Go) are compared in the same suite, while Python/TypeScript are interpreter/JIT-oriented baselines.
- Channel and scheduler semantics differ by runtime implementation; cross-language ratios in concurrency cases should be interpreted with this caveat.
- Host context matters: WSL2 and native Linux can differ on scheduler and timing behavior.

## Budget Gate

- status: `pass`
- no budget issues detected

