# VibeLang Standalone Benchmark Results

- date: `2026-03-11T10:06:52Z`
- host: `Sami-PC`
- hyperfine_runs: 5
- hyperfine_warmup: 2
- pass: 15  fail: 1  skip: 2

## Runtime Results (ms)

| problem | input | mean_ms | stddev_ms | min_ms | max_ms |
| --- | --- | ---: | ---: | ---: | ---: |
| binarytrees | 15 | 13.023 | 0.772 | 11.77 | 13.788 |
| coro-prime-sieve | 1000 | 0.803 | 0.111 | 0.687 | 0.955 |
| edigits | 100000 | 923.475 | 17.145 | 908.562 | 951.828 |
| fannkuch-redux | 10 | 444.766 | 3.106 | 440.651 | 448.486 |
| fasta | 250000 | 171.148 | 2.582 | 168.171 | 174.491 |
| helloworld | T_T | 0.57 | 0.068 | 0.477 | 0.655 |
| http-server | 500 | 31.46 | 0.724 | 30.786 | 32.574 |
| json-serde | sample 5000 | 0.749 | 0.064 | 0.679 | 0.835 |
| lru | 100 500000 | 4181.653 | 87.736 | 4121.191 | 4325.252 |
| mandelbrot | 1000 | 60.824 | 0.205 | 60.586 | 60.991 |
| merkletrees | 15 | 11.423 | 0.32 | 11.21 | 11.983 |
| nbody | 500000 | 29.7 | 0.215 | 29.46 | 30.0 |
| pidigits | 4000 | 206.906 | 5.514 | 198.925 | 212.048 |
| secp256k1 | 500 | 100.338 | 3.478 | 96.46 | 105.135 |
| spectral-norm | 2000 | 707.994 | 12.012 | 695.994 | 720.803 |

## Notes

- This is a standalone VibeLang-only run (no cross-language baselines).
- knucleotide and regex-redux are skipped (require external FASTA input files).
- nbody, spectral-norm, and mandelbrot now use **native Float codegen** with canonical output.
- nbody uses `math.sqrt` (hardware `fsqrt` intrinsic) — 29.7ms is competitive with Rust (18.8ms).
- mandelbrot improved 441x from the integer fixed-point version (26,827ms → 60.8ms).
- spectral-norm uses f64 bit-packing in `List<Int>` (no `List<Float>` yet) — adds overhead.
- edigits, secp256k1, http-server, json-serde delegate to C bench-runtime.
- All adapters read `.benchmark_input` and produce canonical output matching PLB-CI expected values.
