# VibeLang Optimization Priority Report

- baseline_report: `reports/benchmarks/cross_lang/full/results.json`
- latest_summary: `reports/benchmarks/cross_lang/latest/summary.md`
- detailed_summary: `reports/benchmarks/cross_lang/analysis/20260224_100122Z_detailed_summary.md`
- objective: maximize geomean gain versus C/Rust/Go by attacking highest-yield runtime/compiler bottlenecks first while preserving correctness and determinism.

## Current Baseline (Canonical)

Source: `reports/benchmarks/cross_lang/full/summary.md` generated `2026-02-24T09:53:49Z`.

- geomean vs C: `1.831x`
- geomean vs Rust: `1.370x`
- geomean vs Go: `1.826x`
- geomean vs Python: `0.118x`
- geomean vs TypeScript: `0.138x`

Key per-case ratios:

- `hashmap_frequency`: Vibe/C `11.323x`, Vibe/Rust `5.754x`, Vibe/Go `4.388x`
- `hashmap_frequency_int_key`: Vibe/C `3.467x`, Vibe/Rust `1.705x`, Vibe/Go `1.300x`
- `hashmap_frequency_str_key`: Vibe/C `1.026x`, Vibe/Rust `1.880x`, Vibe/Go `2.006x`
- `channel_pingpong`: Vibe/C `0.985x`, Vibe/Rust `0.962x`, Vibe/Go `181.697x`
- `string_concat_checksum`: Vibe/C `2.611x`, Vibe/Rust `1.741x`, Vibe/Go `1.318x`
- `json_roundtrip`: Vibe/C `1.871x`, Vibe/Rust `1.044x`, Vibe/Go `0.496x`

## Baseline-to-Current Delta (Program-Level)

Reference baseline before optimization phases: `2026-02-24T03:31:23Z`.

- geomean vs C: `2.623` -> `1.831` (`-30.19%`)
- geomean vs Rust: `1.909` -> `1.370` (`-28.23%`)
- geomean vs Go: `2.752` -> `1.826` (`-33.65%`)
- `hashmap_frequency` vs C: `146.571` -> `11.323` (`-92.28%`)
- `string_concat_checksum` vs C: `3.828` -> `2.611` (`-31.79%`)
- `json_roundtrip` vs C: `2.265` -> `1.871` (`-17.39%`)
- `channel_pingpong` vs Go: `182.018` -> `181.697` (`-0.18%`, still major hotspot)

## Completed So Far (Infrastructure)

- Cross-language suite is running with `vibelang`, `c`, `rust`, `go`, `python`, `typescript`.
- Collector/validator generate reproducible `quick`, `full`, and `latest` result artifacts.
- CI automation exists for scheduled and on-demand benchmark execution.
- Delta/trend reporting, budgets, rerun warnings, and triage/rollback templates are in place.
- Compiler build phase timing artifacts are emitted per build (`*.compile_phases.json`).

## Current Status Snapshot

| Priority | Area | Status | Notes |
| --- | --- | --- | --- |
| P0 | Map runtime redesign | Completed | Hash-backed runtime map backend is live with counters. |
| P1 | Channel fast path | Completed (partial impact) | Fast-path and counters landed; Go gap remains large. |
| P2 | String/number conversions | Completed | Parse/stringify/minify fast paths and counters landed. |
| P3 | JSON utility tuning | Completed | Validation and minify paths optimized with common-case bypass. |
| P4 | Compiler/codegen throughput | Completed | Phase timing and runtime object caching landed. |

## Priority Ranking (Highest Impact First)

## P0: Map Data Structure Redesign (`Map<Int,Int>`, `Map<Str,Int>`)

**Why this is first**

- `hashmap_frequency` is the largest algorithmic regression:
  - Vibe/C: `146.571x`
  - Vibe/Rust: `82.742x`
  - Vibe/Go: `64.125x`
- Runtime map ops in `runtime/native/vibe_runtime.c` are linear scan (`O(n)` lookup/update):
  - `vibe_map_set_i64_i64`, `vibe_map_get_i64_i64`, `vibe_map_contains_i64_i64`
  - `vibe_map_set_str_i64`, `vibe_map_get_str_i64`, `vibe_map_contains_str_i64`

**Optimization direction**

- Replace linear-entry arrays with open-addressing hash tables (Robin Hood style probing).
- Keep deterministic iteration semantics via explicit policy (stable order index if required).
- Add load-factor policy, resize thresholds, and collision instrumentation.
- Cache string-key hashes to reduce repeated `strcmp` pressure.

**Expected impact**

- Largest single geomean improvement opportunity.
- Target benchmark improvement:
  - <= `15x` vs C (intermediate),
  - <= `6x` vs C (longer-term).

## P1: Channel Fast Path and Contention Reduction

**Why this is second**

- `channel_pingpong` consumes `89.79%` of VibeLang suite time share.
- Current channel path (`vibe_chan_send_i64`, `vibe_chan_recv_i64`) uses mutex+condvar for every operation.
- Result: close to C/Rust pthread lane, but far from Go runtime channels:
  - Vibe/Go: `182.018x`.

**Optimization direction**

- Introduce uncontended send/recv fast paths.
- Add SPSC and then MPSC specialization where safe.
- Reduce context-switch churn (batched wakeups and spin-then-park policy).
- Add channel contention counters for tuning loops.

**Expected impact**

- Massive absolute runtime reduction for concurrency-latency class.
- Target benchmark improvement:
  - >= `3x` first-pass reduction in `channel_pingpong`.

## P2: String and Numeric Conversion Pipeline (`string_concat_checksum`)

**Why this is third**

- `string_concat_checksum` remains behind:
  - Vibe/C: `3.828x`
  - Vibe/Rust: `2.775x`
  - Vibe/Go: `1.947x`
- Conversion loops are allocation and parse heavy.

**Optimization direction**

- Add fast integer parse/stringify paths with lower allocation pressure.
- Reuse scratch buffers safely in hot conversion loops.
- Add microbench coverage for conversion primitives.

**Expected impact**

- Medium-high geomean benefit after P0/P1.
- Target benchmark improvement:
  - >= `40%` runtime reduction for `string_concat_checksum`.

## P3: JSON Utility Micro-Optimizations (`json_roundtrip`)

**Why this is fourth**

- `json_roundtrip` gap is moderate:
  - Vibe/C: `2.265x`
  - Vibe/Rust: `1.255x`
  - Vibe/Go: `0.614x` (already faster than Go on this case)
- Existing JSON helpers are functionally stable, so this is a lower-risk optimization lane.

**Optimization direction**

- Add minifier fast path for already-minified payloads.
- Tighten common-path validation checks.
- Reduce repeated recomputation in deterministic payload loops.

**Expected impact**

- Medium impact, lower implementation risk.
- Target:
  - `20-30%` runtime reduction in this case.

## P4: Compiler + Codegen Throughput Polish

**Why this matters**

- Compile latency is serviceable but behind C/Go:
  - Vibe mean compile: `353.13 ms`
  - C: `95.88 ms`
  - Go: `114.75 ms`
  - Rust: `360.00 ms`

**Optimization direction**

- Add compiler phase timing (parse/type/lower/codegen/link).
- Remove top redundant allocations/traversals.
- Add throughput guardrails for benchmark corpus compiles.

**Expected impact**

- Better developer loop and CI throughput.
- Secondary to runtime hotspots for runtime geomean competitiveness.

## Priority-to-Impact Matrix

| Priority | Area | Perf impact on current suite | Engineering complexity | Suggested start |
| --- | --- | --- | --- | --- |
| P0 | Map runtime redesign | Very high | High | Immediate |
| P1 | Channel fast path | Very high | High | Immediate (parallel with P0) |
| P2 | String/number conversions | Medium-high | Medium | After P0/P1 in flight |
| P3 | JSON utility tuning | Medium | Low-medium | After P2 |
| P4 | Compiler/codegen polish | Medium (compile), low (runtime) | Medium | Ongoing lane |

## Progress Tracking

| Priority | Current | Intermediate target | Long-term target | Progress state |
| --- | --- | --- | --- | --- |
| P0 map | `146.571x` vs C | `<15x` vs C | `<6x` vs C | Not started |
| P1 channel | `182.018x` vs Go | `<25x` vs Go | `<10x` vs Go | Not started |
| P2 conversion | `3.828x` vs C | `<2.2x` vs C | `<1.5x` vs C | Not started |
| P3 JSON | `2.265x` vs C | `<1.8x` vs C | `<1.3x` vs C | Not started |
| P4 compile | `353.13 ms` | `<300 ms` | `<260 ms` | Not started |

## Recommendation

Completed implementation tracks and current focus:

1. **Continue channel-focused optimization** (still dominant absolute time and largest Go-relative gap)
2. **Tighten worker pool variance** (rerun warnings occasionally trigger)
3. **Track compile throughput trend** with phase-timing reports and runtime cache hit rates

All phases are now implemented; remaining work is iterative tuning and stricter budget ratcheting over time.

