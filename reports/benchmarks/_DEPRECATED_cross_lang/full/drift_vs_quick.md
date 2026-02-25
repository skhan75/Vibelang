# Cross-Language Profile Drift

- current_profile: `full`
- comparison_profile: `quick`
- generated_at_utc: `2026-02-24T09:53:49Z`
- shared_case_count: `10`

## Geomean Drift

| baseline | before | after | delta_abs | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| c | 2.046 | 1.831 | -0.215 | -10.51% |
| rust | 1.546 | 1.370 | -0.176 | -11.39% |
| go | 2.114 | 1.826 | -0.288 | -13.63% |
| python | 0.131 | 0.118 | -0.013 | -9.74% |
| typescript | 0.158 | 0.138 | -0.020 | -12.72% |

## Per-Case Vibe Drift

| case | before_ms | after_ms | delta_abs_ms | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| fib_recursive | 1.400 | 1.900 | 0.500 | 35.71% |
| prime_sieve | 1.800 | 1.800 | 0.000 | 0.00% |
| sort_int | 2.000 | 2.000 | 0.000 | 0.00% |
| hashmap_frequency | 25.200 | 17.550 | -7.650 | -30.36% |
| hashmap_frequency_int_key | 5.000 | 5.200 | 0.200 | 4.00% |
| hashmap_frequency_str_key | 23.800 | 17.950 | -5.850 | -24.58% |
| string_concat_checksum | 10.800 | 7.050 | -3.750 | -34.72% |
| json_roundtrip | 11.400 | 9.450 | -1.950 | -17.11% |
| channel_pingpong | 2440.800 | 2462.000 | 21.200 | 0.87% |
| worker_pool_reduce | 1.800 | 1.250 | -0.550 | -30.56% |

Interpretation: for runtime deltas in this table, negative delta is improvement (faster).

