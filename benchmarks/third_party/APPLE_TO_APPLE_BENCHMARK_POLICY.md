# Apples-to-Apples Benchmark Policy

This policy defines the minimum standard for benchmark results that can be
shared publicly (research, blogs, talks, release notes).

## Non-negotiable requirements

1. Same workload semantics across languages:
   - No proxies, stubs, canned outputs, or precomputed output shortcuts.
   - Each language implementation must execute equivalent algorithmic work for
     the same input data.
2. Same benchmark harness and inputs:
   - Use canonical PLB-CI problem inputs and workload definitions.
   - No language-specific workload simplification.
3. Same environment class:
   - Docker-backed reproducible runs only for public claims.
   - Fixed host profile (dedicated VM or dedicated bare-metal).
4. Full matrix completeness:
   - No missing required language lanes for publication mode.
   - No "shared-subset only" claims presented as full-language conclusions.
5. Reproducible provenance:
   - Pin upstream benchmark suite by commit (not floating branch tip).
   - Record toolchain versions, host metadata, and run timestamp IDs.
6. Strict execution mode:
   - Publication runs must not use `--no-docker`.
   - Publication runs must not use `--allow-preflight-degraded`.
   - Publication runs must not use permissive missing-lane behavior.

## Current blockers (must be resolved before publication)

### B1. VibeLang adapter parity (Reopened → Partially Closed)

**2026-03-10 audit** found 8 of 18 adapters used hardcoded problem sizes and
fake/simplified algorithms. All adapters have been rewritten to read
`.benchmark_input` and use canonical problem sizes.

Current status:
- 11 adapters are fully canonical (correct algorithm, correct output).
- 4 adapters are runtime-backed (edigits, secp256k1, http-server, json-serde).
- 3 adapters use integer-approximation (nbody, spectral-norm, mandelbrot)
  because Float codegen is not yet functional. These produce equivalent workloads
  but their output does not match canonical floating-point values.

Exit criterion for full closure:
- Float codegen lands, enabling canonical f64 output for nbody, spectral-norm,
  and mandelbrot.

Evidence:
- `benchmarks/third_party/plbci/adapters/vibelang/PARITY_MANIFEST.yaml` (v2)
- `reports/benchmarks/third_party/vibelang_standalone_results.json`

### B2. Runtime/compile matrix is incomplete in current host runs (Open)

Recent runs have missing required lanes in the generated report.

Exit criterion:
- All required runtime and compile lanes report `status=ok` in strict run
  results for publication profile.

### B3. Docker reproducibility (Closed)

Docker-backed full run completed successfully on 2026-03-11.

Closure evidence:
- `reports/benchmarks/third_party/full/results.json` (2026-03-11T08:09:15Z)
- Docker 28.0.1 on WSL2, AMD Ryzen 9 5900X, 31.3 GiB RAM
- All 10 languages built and benchmarked (Swift unavailable in image)

### B4. Upstream benchmark suite ref is floating (Closed)

`language_matrix.json` now pins `plbci_ref` to immutable commit:
`ad18b203dd1769724f4eea94fc3ac1e99f6593e0`.

Closure evidence:
- `benchmarks/third_party/plbci/config/language_matrix.json`
- `tooling/metrics/collect_third_party_benchmarks.py` publication metadata

### B5. Publication gating still permits permissive execution paths (Closed)

Publication mode now hard-fails degraded/permissive execution paths.

Closure evidence:
- `tooling/metrics/collect_third_party_benchmarks.py`
- `tooling/metrics/validate_third_party_benchmarks.py`
- `tooling/metrics/compare_third_party_benchmarks.py`
- `tooling/metrics/validate_adapter_parity.py`

### B6. Runtime feature gaps block full canonical parity (Closed)

Canonical parity required adding runtime/stdlib capabilities:
- `math.edigits` (high-precision digits of e output)
- `net.*` and `http.server_bench` (minimal socket-backed HTTP parity path)
- `crypto.secp256k1_bench` (field arithmetic + scalar multiplication output)
- `json.canonical` + `hash.md5_hex` (serde workload verification)

Closure evidence:
- Runtime/stdlib surfaces in `stdlib/` and the updated parity manifest.

### B7. Go and Kotlin baseline data is corrupted (Open)

Both the 2026-03-01 and 2026-03-11 Docker runs produced ~1.1-1.9ms for every
Go problem and ~1.1-1.6ms for every Kotlin problem, regardless of complexity.
Go nbody(500000) at 1.2ms is physically impossible. The PLB-CI harness is
failing to capture real execution timing for these languages.

Exit criterion:
- Diagnose root cause in PLB-CI BenchTool (likely test/bench step not running).
- Re-run with verbose logging and verify Go/Kotlin produce realistic times.
- Validate that Go binarytrees(15) > 50ms, Go nbody(500000) > 10ms, etc.

### B8. Float codegen blocks canonical parity for 3 benchmarks (Open)

VibeLang's Float type exists but native codegen fails with cranelift type
mismatch errors. This blocks canonical implementations of:
- nbody (requires f64 physics simulation)
- spectral-norm (requires f64 matrix-vector operations)
- mandelbrot (requires f64 complex arithmetic)

Current workaround: integer-approximation adapters that perform equivalent
O(N) workloads but produce different output.

Exit criterion:
- `vibe build` succeeds for programs using Float arithmetic.
- All three adapters rewritten with f64 and producing canonical output.

## Publication checklist

Only mark benchmark evidence as public-ready when all of the following are true:

The canonical checklist for benchmark execution + publication readiness is:

- `docs/checklists/benchmarks.md`

## Current status

- Publication status: `blocked` (B7, B8 open)
- Shareability: VibeLang vs Rust/C/C++/Zig/Python/TS comparisons are honest and can be cited with caveats about Float codegen and runtime-backed adapters
- Last full run: 2026-03-11T08:09:15Z (Docker-backed, `full` profile)
