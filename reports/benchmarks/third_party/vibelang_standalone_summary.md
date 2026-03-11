# VibeLang Benchmark Results — Docker Production Run

- date: `2026-03-11T17:48:10Z`
- host: `Docker (AMD Ryzen 9 5900X, 24 cores, 31.3 GiB)`
- platform: `Linux 6.6.87.2-microsoft-standard-WSL2 x86_64`
- tool: `PLB-CI BenchTool + hyperfine`
- binary: `vibe 1.0.2 (release, bench-runtime, GMP-enabled)`
- languages_tested: 12 (VibeLang, C, C++, Rust, Zig, Go, Kotlin, Swift, Python, TypeScript, PHP, Elixir)
- languages_valid: 9 (Go/Kotlin/Swift had build failures)

## VibeLang Runtime Results (median ms)

| problem | median_ms | samples | status |
| --- | ---: | ---: | --- |
| binarytrees | 13.9 | 1 | OK |
| coro-prime-sieve | 3.3 | 1 | OK |
| edigits | 875.2 | 1 | OK |
| fannkuch-redux | 332.8 | 1 | OK |
| fasta | 29.8 | 1 | OK |
| helloworld | 1.7 | 2 | OK |
| http-server | 36.5 | 1 | OK |
| json-serde | 3.3 | 1 | OK |
| knucleotide | 0.0 | 0 | FAILED |
| lru | 3.8 | 1 | OK |
| mandelbrot | 59.1 | 1 | OK |
| merkletrees | 14.5 | 1 | OK |
| nbody | 37.2 | 1 | OK |
| nsieve | 1.7 | 1 | OK |
| pidigits | 201.5 | 1 | OK |
| regex-redux | 4592.4 | 1 | OK |
| secp256k1 | 90.9 | 1 | OK |
| spectral-norm | 565.0 | 1 | OK |

## Cross-Language Geomean Ratios (VibeLang / Other)

| vs Language | Geomean | Shared | Interpretation |
| --- | ---: | ---: | --- |
| C | 0.89x | 3 | VibeLang 1.1x faster |
| C++ | 1.74x | 4 | VibeLang 1.7x slower |
| Rust | 0.93x | 16 | VibeLang 1.1x faster |
| Zig | 0.82x | 12 | VibeLang 1.2x faster |
| Python | 0.05x | 3 | VibeLang 20.6x faster |
| TypeScript | 0.12x | 12 | VibeLang 8.6x faster |
| PHP | 0.04x | 3 | VibeLang 25.0x faster |
| Elixir | 0.03x | 5 | VibeLang 37.0x faster |

## Per-Problem Ratios vs Rust (16 shared benchmarks)

| problem | VibeLang | Rust | ratio | winner |
| --- | ---: | ---: | ---: | --- |
| nsieve | 1.7 | 60.0 | 0.028x | VibeLang 35.3x faster |
| json-serde | 3.3 | 56.2 | 0.059x | VibeLang 17.0x faster |
| coro-prime-sieve | 3.3 | 50.3 | 0.066x | VibeLang 15.2x faster |
| merkletrees | 14.5 | 107.4 | 0.135x | VibeLang 7.4x faster |
| binarytrees | 13.9 | 86.1 | 0.161x | VibeLang 6.2x faster |
| lru | 3.8 | 20.0 | 0.190x | VibeLang 5.3x faster |
| http-server | 36.5 | 192.5 | 0.190x | VibeLang 5.3x faster |
| secp256k1 | 90.9 | 142.0 | 0.640x | VibeLang 1.6x faster |
| pidigits | 201.5 | 226.6 | 0.889x | VibeLang 1.1x faster |
| helloworld | 1.7 | 1.2 | 1.417x | Rust 1.4x faster |
| fasta | 29.8 | 15.3 | 1.948x | Rust 1.9x faster |
| nbody | 37.2 | 18.5 | 2.011x | Rust 2.0x faster |
| mandelbrot | 59.1 | 11.7 | 5.051x | Rust 5.1x faster |
| fannkuch-redux | 332.8 | 19.9 | 16.724x | Rust 16.7x faster |
| edigits | 875.2 | 39.4 | 22.213x | Rust 22.2x faster |
| regex-redux | 4592.4 | 43.9 | 104.611x | Rust 104.6x faster |

**VibeLang wins 9 of 16 benchmarks vs Rust.**

## Compile-Time (helloworld cold build)

| Language | Cold (ms) | Binary Size |
| --- | ---: | ---: |
| VibeLang | 3,023 | 328 KB |
| C | 1,533 | 16 KB |
| C++ | 1,529 | 16 KB |
| Rust | 4,565 | 316 KB |
| Zig | 5,976 | 1.7 MB |

VibeLang compiles 1.5x faster than Rust and 2.0x faster than Zig.

## Notes

- Go, Kotlin, and Swift benchmarks had build failures (binaries not found at expected PLB-CI paths).
- VibeLang knucleotide adapter failed (0 samples).
- regex-redux is dominated by POSIX regex overhead (4.6s vs Rust's 44ms).
- edigits uses custom bigint (GMP was installed but may not have linked correctly).
- Python results are limited (3 valid benchmarks) because many Python benchmarks use C extensions (NumPy/BLAS).
- C/C++ have limited overlap (3-5 shared benchmarks) — geomean is not fully representative.
