# Cross-Language Profile Drift

- current_profile: `quick`
- comparison_profile: `full`
- generated_at_utc: `2026-02-24T09:59:48Z`
- shared_case_count: `10`

## Geomean Drift

| baseline | before | after | delta_abs | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| c | 1.831 | 1.750 | -0.082 | -4.46% |
| rust | 1.370 | 1.389 | 0.019 | 1.37% |
| go | 1.826 | 1.755 | -0.071 | -3.88% |
| python | 0.118 | 0.120 | 0.002 | 1.68% |
| typescript | 0.138 | 0.144 | 0.006 | 4.31% |

## Per-Case Vibe Drift

| case | before_ms | after_ms | delta_abs_ms | delta_pct |
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

Interpretation: for runtime deltas in this table, negative delta is improvement (faster).

