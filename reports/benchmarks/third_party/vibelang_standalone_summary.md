# VibeLang Standalone Benchmark Results

- date: `2026-03-11T10:30:00Z`
- host: `Sami-PC`
- hyperfine_runs: 5
- hyperfine_warmup: 2
- pass: 15  fail: 0  skip: 2

## Runtime Results (ms)

| problem | input | mean_ms | stddev_ms | min_ms | max_ms | vs_prev |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| binarytrees | 15 | 12.0 | 0.2 | 11.8 | 12.2 | ~same |
| coro-prime-sieve | 1000 | 0.8 | 0.1 | 0.7 | 1.0 | ~same |
| edigits | 100000 | 901.4 | 8.8 | 893 | 910 | 1.02x faster |
| fannkuch-redux | 10 | 327.2 | 1.1 | 326 | 329 | **1.36x faster** |
| fasta | 250000 | 27.0 | 0.9 | 26.1 | 28.0 | **6.3x faster** |
| helloworld | T_T | 0.58 | 0.1 | 0.48 | 0.66 | ~same |
| http-server | 500 | 31.5 | 0.7 | 30.8 | 32.6 | ~same |
| json-serde | sample 5000 | 0.75 | 0.06 | 0.68 | 0.84 | ~same |
| lru | 100 500000 | 244.1 | 3.7 | 240 | 248 | **17.1x faster** |
| mandelbrot | 1000 | 54.0 | 0.8 | 53.2 | 55.0 | 1.13x faster |
| merkletrees | 15 | 12.0 | 0.3 | 11.7 | 12.3 | ~same |
| nbody | 500000 | 32.0 | 2.8 | 29.2 | 35.0 | ~same |
| pidigits | 4000 | 206.9 | 5.5 | 199 | 212 | ~same |
| secp256k1 | 500 | 100.3 | 3.5 | 96.5 | 105 | ~same |
| spectral-norm | 500 | 36.8 | 0.2 | 36.6 | 37.0 | **19.2x faster** |

## Changes Since Last Run

### Phase 1: LRU Adapter Rewrite
- Replaced O(mod) linear-scan eviction with FIFO queue + timestamp map (amortized O(1))
- **LRU: 4182ms -> 244ms (17.1x faster)**

### Phase 2: GMP Bigint Support
- Added GMP-backed edigits behind `VIBE_USE_GMP` compile flag (activates in Docker)
- Karatsuba fallback added to custom bigint for non-GMP environments
- Docker build now installs `libgmp-dev` and links `-lgmp`

### Phase 3: String Builder for Fasta
- Exposed `str_builder` as a VibeLang stdlib namespace (`str_builder.new`, `append_char`, `finish`)
- Rewrote fasta adapter to use `str_builder.append_char` instead of `line = line + ch`
- **Fasta: 171ms -> 27ms (6.3x faster)** — eliminated O(n^2) string concatenation

### Phase 4: Inline list.set
- `list.set(i, val)` now emits inline Cranelift store (matching inline list read pattern)
- Falls back to FFI for Map types via runtime tag check
- **Fannkuch-redux: 445ms -> 327ms (1.36x faster)**

### Phase 5: Zero-cost f64 Bitcast
- `convert.f64_to_bits` / `convert.f64_from_bits` now emit Cranelift `bitcast` instead of FFI calls
- Eliminates function call overhead for float bit-packing in `List<Int>`
- **Spectral-norm: 708ms -> 37ms (19.2x faster)**

### Phase 6: SIMD Intrinsics
- Added `simd.f64x2_splat`, `f64x2_make`, `f64x2_add`, `f64x2_sub`, `f64x2_mul`, `f64x2_gt`, `f64x2_extract`
- Mandelbrot refactored with function extraction for better MIR optimization
- **Mandelbrot: 61ms -> 54ms (1.13x faster)**

### Phase 7: Docker Infrastructure
- Added PHP to language matrix and Dockerfile
- Fixed Swift binary path detection in Dockerfile
- TMPDIR fix for Go/Kotlin already in place (awaiting Docker verification)

## Notes

- This is a standalone VibeLang-only run (no cross-language baselines).
- knucleotide and regex-redux are skipped (require external FASTA input files).
- nbody, spectral-norm, and mandelbrot use native Float codegen with canonical output.
- spectral-norm uses zero-cost f64 bitcast for `List<Int>` storage (no FFI overhead).
- fasta uses `str_builder.append_char` for O(n) line construction.
- LRU uses FIFO queue with amortized O(1) eviction.
- edigits will use GMP when `libgmp-dev` is available (Docker builds).
- Docker re-run needed for cross-language comparisons (Go, Kotlin, Swift, PHP).
