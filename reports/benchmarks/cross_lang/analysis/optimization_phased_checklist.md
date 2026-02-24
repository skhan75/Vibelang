# VibeLang Performance Optimization Implementation Plan (Phased, Detailed)

This document is the execution playbook for all benchmark-driven optimization work.
It is intentionally detailed so implementation, validation, and rollout are unambiguous.

- baseline_results_full: `reports/benchmarks/cross_lang/full/results.json`
- baseline_results_quick: `reports/benchmarks/cross_lang/quick/results.json`
- baseline_summary_latest: `reports/benchmarks/cross_lang/latest/summary.md`
- detailed_summary: `reports/benchmarks/cross_lang/analysis/20260224_100122Z_detailed_summary.md`
- priority_report: `reports/benchmarks/cross_lang/analysis/optimization_priority_report.md`

## Current Status Dashboard

- last_updated_utc: `2026-02-24T10:01:22Z`
- benchmark_baseline_profile: `full`
- benchmark_languages: `vibelang`, `c`, `rust`, `go`, `python`, `typescript`

### Completed So Far

- [x] Cross-language benchmark suite is operational across 6 languages.
- [x] `quick`, `full`, and `latest` result artifacts are generated and validated.
- [x] CI benchmark workflow is present and publishing benchmark artifacts.
- [x] Analysis docs have aligned baseline references to current summary values.
- [x] Delta reporting, trend reporting, and baseline pointer workflow are active.
- [x] Budget gates, rerun policy warnings, and triage/rollback documentation are active.

### Optimization Phase Status (Implementation)

- [x] Phase 0 Measurement Hardening: completed.
- [x] Phase 1 Runtime Map Algorithmics: completed.
- [x] Phase 2 Map Lowering/Specialization Cleanup: completed.
- [x] Phase 3 Channel Fast Path: completed.
- [x] Phase 4 String/JSON Efficiency: completed.
- [x] Phase 5 Compiler Throughput: completed.
- [x] Phase 6 Regression Gates and Publication: completed.

### Evidence Links (Current)

- [x] Baseline summary: `reports/benchmarks/cross_lang/latest/summary.md`
- [x] Full baseline JSON: `reports/benchmarks/cross_lang/full/results.json`
- [x] Detailed interpretation: `reports/benchmarks/cross_lang/analysis/20260224_100122Z_detailed_summary.md`
- [x] Priority ordering: `reports/benchmarks/cross_lang/analysis/optimization_priority_report.md`
- [x] Delta artifacts: `reports/benchmarks/cross_lang/analysis/deltas/latest_delta.md`
- [x] Budget config: `reports/benchmarks/cross_lang/analysis/performance_budgets.json`

### Measured Program Delta (Baseline -> Current Full)

- [x] geomean vs C: `2.623` -> `1.831` (`-30.19%`)
- [x] geomean vs Rust: `1.909` -> `1.370` (`-28.23%`)
- [x] geomean vs Go: `2.752` -> `1.826` (`-33.65%`)
- [x] `hashmap_frequency` vs C: `146.571` -> `11.323` (`-92.28%`)
- [x] `string_concat_checksum` vs C: `3.828` -> `2.611` (`-31.79%`)
- [x] `json_roundtrip` vs C: `2.265` -> `1.871` (`-17.39%`)
- [x] `channel_pingpong` vs Go: `182.018` -> `181.697` (minimal change; keep tuning lane)

## 0) Program Controls (Do Not Skip)

### Program Objectives

- [ ] Improve runtime competitiveness against C/Rust/Go without sacrificing language correctness guarantees.
- [ ] Preserve deterministic behavior and reproducible benchmarking evidence.
- [ ] Keep performance wins durable by converting them into automated regression gates.

### Non-Negotiable Quality Constraints

- [ ] No optimization lands without correctness tests and benchmark evidence.
- [ ] No benchmark comparison is accepted without environment fingerprint metadata.
- [ ] No cross-language result is published without checksum and ops parity.
- [ ] No runtime fast path lands without a safe fallback path and rollback switch.
- [ ] No compiler throughput optimization lands if deterministic outputs regress.

### Delivery Operating Model

- [ ] Create one phase tracker note per phase under `reports/benchmarks/cross_lang/analysis/phase_logs/` (new file per phase).
- [ ] Require before/after benchmark snapshots for every material optimization PR.
- [ ] Require one reviewer focused on correctness and one reviewer focused on performance methodology.
- [ ] Enforce "small diff sequence" approach for risky runtime refactors (no giant unreviewable drops).
- [ ] Keep phase progress updated in this file weekly.

### Evidence Requirements (Per PR)

- [ ] Link code diff.
- [ ] Link targeted microbench results.
- [ ] Link full-suite `quick` results.
- [ ] Note expected and observed impact on at least one key KPI.
- [ ] Record any benchmark caveat or noise anomaly.

### Phase Dependency Map

- [ ] Phase 0 must complete before Phase 1 through Phase 6 exits are accepted.
- [ ] Phase 1 and Phase 3 may run in parallel after Phase 0 hardening.
- [ ] Phase 2 depends on Phase 1 data structures being stable enough for compiler lowering integration.
- [ ] Phase 4 should begin after first wave wins from Phase 1 and Phase 3.
- [ ] Phase 5 may run as a parallel lane after Phase 0.
- [ ] Phase 6 starts only when Phases 1 to 5 have measurable and validated outcomes.

---

## Phase 0 - Measurement Hardening (Foundation)

### Primary Outcome

Establish statistically reliable and reproducible performance measurement so optimization decisions are trustworthy.

### Scope

- In scope: benchmark harness, report schema, environment normalization, variance tracking, fairness annotations.
- Out of scope: runtime algorithm redesigns (handled in later phases).

### Detailed Implementation Checklist

#### A. Environment Capture and Reproducibility

- [ ] Extend collector environment metadata to include CPU governor policy.
- [ ] Capture physical and logical core counts (if available separately).
- [ ] Capture NUMA layout (if available) and record as optional metadata field.
- [ ] Capture memory size and swap status.
- [ ] Capture compiler/runtime tool versions (`vibe`, `gcc`, `rustc`, `go`, `python3`, `node`, `tsc`).
- [ ] Capture git revision for benchmark suite and toolchain revision hash.
- [ ] Record benchmark launch command and profile parameters for every run.
- [ ] Add reproducibility runbook under `reports/benchmarks/cross_lang/analysis/` with exact command sequence.

#### B. Benchmark Statistic Hardening

- [ ] Add p99 wall time to runtime summary schema.
- [ ] Add median absolute deviation (MAD) or equivalent robust variance metric.
- [ ] Add relative standard deviation flag for noisy cases.
- [ ] Add outlier clipping policy (document-only first, optional code later).
- [ ] Add confidence comment field in generated markdown summary.
- [ ] Ensure summary generation supports dynamic baseline languages cleanly.

#### C. Trend and Drift Tracking

- [ ] Add quick-vs-full drift report generation by case and geomean.
- [ ] Add rolling comparison support against previous baseline run artifact.
- [ ] Add "regression suspected" heuristic when drift exceeds configured threshold.
- [ ] Emit machine-readable trend JSON for CI and dashboard consumption.

#### D. Fairness and Methodology Notes

- [ ] Add "fairness notes" section template to generated summary.
- [ ] Document per-case caveats (runtime model mismatch, scheduler semantics, etc.).
- [ ] Document interpreter-vs-native caveat notes for Python/TypeScript.
- [ ] Document host-specific caveats (WSL2 vs native Linux).

#### E. CI/Automation Readiness

- [ ] Add native Linux lane in automation (non-WSL2 authoritative lane).
- [ ] Ensure scheduled full runs publish both results and methodology metadata.
- [ ] Validate artifact retention policy is sufficient for trend analysis windows.

### Validation and Tests

- [ ] Schema validation tests for new summary fields.
- [ ] Backward compatibility test on older result artifacts.
- [ ] Generate two consecutive full runs on same host and verify variance constraints.

### Risks and Mitigations

- [ ] Risk: high noise in concurrency cases masks real wins. Mitigation: enforce p95/p99 and multiple reruns.
- [ ] Risk: host drift invalidates comparisons. Mitigation: strict metadata gating and pinned lane policy.

### Exit Criteria and Required Artifacts

- [ ] Two consecutive native-Linux full runs are within +/-5% for non-concurrency microcases.
- [ ] Reproducibility runbook is published and reviewed.
- [ ] New summary schema (including p99) is live and validated.
- [ ] Fairness notes appear in generated summaries.

---

## Phase 1 - Runtime Map Algorithmics (P0)

### Primary Outcome

Replace linear-scan map behavior with hash-based map backends to eliminate algorithmic bottlenecks in map-heavy workloads.

### Scope

- In scope: runtime map internals for `Map<Int,Int>` and `Map<Str,Int>`, hash/resize policy, correctness/perf validation.
- Out of scope: compiler lowering cleanup (handled in Phase 2).

### Detailed Implementation Checklist

#### A. Data Structure and API Design

- [ ] Finalize map backend strategy (open addressing with robin hood probing recommended).
- [ ] Define entry layout for int-key and str-key maps.
- [ ] Define tombstone semantics for delete support.
- [ ] Define load factor thresholds and resize growth policy.
- [ ] Define stable API surface compatibility for existing map ops.
- [ ] Define deterministic iteration behavior (preserve order or explicitly document change).

#### B. Runtime Implementation (Core)

- [ ] Implement hash table backend for `Map<Int,Int>`.
- [ ] Implement hash table backend for `Map<Str,Int>`.
- [ ] Implement `set/get/contains/remove/len/key_at` against new backend.
- [ ] Add safe memory ownership handling for string keys.
- [ ] Add resize and rehash routines with bounded failure modes.
- [ ] Add collision probe accounting in debug/perf mode.
- [ ] Add fallback path behind feature toggle for rollback safety.

#### C. Runtime Correctness and Safety

- [ ] Verify behavior parity with existing map semantics.
- [ ] Validate zero/negative key handling and boundary integer behavior.
- [ ] Validate duplicate key overwrite semantics.
- [ ] Validate delete then reinsert behavior.
- [ ] Validate iteration behavior under mutation constraints.
- [ ] Run leak checks and allocator stress checks for string-key maps.

#### D. Performance Engineering

- [ ] Add map microbench suite: insertion-heavy, read-heavy, mixed RW, delete-heavy.
- [ ] Benchmark cardinality tiers (tiny, small, medium, large).
- [ ] Add synthetic collision stress tests.
- [ ] Capture CPU, RSS, and branch-sensitive behavior where possible.
- [ ] Run full cross-language suite and compute geomean effect.

#### E. Rollout Plan

- [ ] Land backend in guarded mode first (opt-in toggle).
- [ ] Run validation and stress suite in both guarded and fallback modes.
- [ ] Flip default once parity and perf gates pass.
- [ ] Keep rollback flag available for one release cycle.

### Validation and Tests

- [ ] Unit tests for each map operation and edge case.
- [ ] Property tests: operation sequences preserve reference map behavior.
- [ ] Fuzz tests on randomized operation streams.
- [ ] Cross-language benchmark parity checks remain green.

### Risks and Mitigations

- [ ] Risk: semantic drift in iteration behavior. Mitigation: explicit contract and tests.
- [ ] Risk: resize regression causes latency spikes. Mitigation: staged resize policy and counters.
- [ ] Risk: string key memory bugs. Mitigation: ownership tests and stress/leak checks.

### Exit Criteria and Required Artifacts

- [ ] `hashmap_frequency` improves by at least `10x` from current VibeLang baseline.
- [ ] No regressions in map correctness tests.
- [ ] Geomean versus C/Rust/Go improves materially (target >=20% relative reduction vs current baseline).
- [ ] Phase report includes before/after map microbench and full-suite evidence.

---

## Phase 2 - Map Lowering and Type-Specialization Cleanup

### Primary Outcome

Ensure compiler/runtime integration preserves map specialization so int-key operations stay on fast paths without accidental conversion overhead.

### Scope

- In scope: compiler diagnostics, lowering rules, specialization guarantees, benchmark fixtures.
- Out of scope: channel runtime redesign (Phase 3).

### Detailed Implementation Checklist

#### A. Compiler Front-End and Type System

- [ ] Audit map literal typing and key type inference paths.
- [ ] Add explicit compiler checks for mixed-key map construction patterns.
- [ ] Add diagnostics for patterns that force slow path fallback.
- [ ] Add warning class for implicit conversion in hot map contexts.

#### B. Lowering and Codegen Bridge

- [ ] Audit IR lowering from typed map operations to runtime calls.
- [ ] Ensure `Map<Int,Int>` lowers to int-specialized runtime API only.
- [ ] Ensure `Map<Str,Int>` lowers to str-specialized runtime API only.
- [ ] Remove accidental conversion bridges in hot loops.
- [ ] Add internal assert hooks in debug builds for specialization mismatch.

#### C. Runtime Call Surface Cleanup

- [ ] Validate runtime API signatures are specialization-friendly and stable.
- [ ] Reduce dynamic dispatch overhead where static specialization is possible.
- [ ] Add lightweight counters for specialization hit/miss events.

#### D. Benchmark and Fixture Enhancements

- [ ] Add explicit int-key map benchmark case variant.
- [ ] Add explicit str-key map benchmark case variant.
- [ ] Add "mixed misuse" fixture to validate diagnostics and fallback behavior.
- [ ] Ensure report output breaks out int-key and str-key map results separately.

### Validation and Tests

- [ ] Compiler tests for diagnostics and inference behavior.
- [ ] Lowering tests asserting expected runtime call targets.
- [ ] End-to-end map benchmark cases with specialization coverage.

### Risks and Mitigations

- [ ] Risk: diagnostics become noisy. Mitigation: keep warnings targeted to perf-significant cases.
- [ ] Risk: hidden fallback survives. Mitigation: debug assert + counters + fixture coverage.

### Exit Criteria and Required Artifacts

- [ ] `hashmap_frequency` no longer requires workaround patterns to avoid fallback.
- [ ] int-key and str-key map performance are separately reported in benchmark outputs.
- [ ] Specialization miss rate is documented and near-zero in target workloads.

---

## Phase 3 - Channel Fast Path and Scheduling Efficiency (P1)

### Primary Outcome

Reduce channel send/recv latency and contention overhead, especially for ping-pong and fan-in patterns.

### Scope

- In scope: channel runtime path redesign, uncontended fast path, optional queue specializations, scheduler interactions.
- Out of scope: non-channel scheduler redesign unrelated to benchmark bottlenecks.

### Detailed Implementation Checklist

#### A. Channel Path Architecture

- [ ] Define uncontended send/recv fast path with minimal locking.
- [ ] Define fallback slow path for contended scenarios.
- [ ] Add optional SPSC ring buffer mode for low-overhead single producer/single consumer.
- [ ] Add optional MPSC optimized enqueue path for fan-in patterns.
- [ ] Define bounded-memory behavior and backpressure semantics.

#### B. Runtime Implementation

- [ ] Implement fast path in `vibe_chan_send_i64`.
- [ ] Implement fast path in `vibe_chan_recv_i64`.
- [ ] Implement contention-aware handoff policy.
- [ ] Add spin-then-park policy with tunable thresholds.
- [ ] Add wakeup batching policy and avoid unnecessary signal storms.
- [ ] Keep robust fallback path for correctness-first behavior.

#### C. Instrumentation and Observability

- [ ] Add counters: wakeups, parks, spin loops, slow-path entries, queue depth high-watermark.
- [ ] Add optional trace hooks for latency decomposition.
- [ ] Emit channel perf diagnostics in debug/perf mode.

#### D. Correctness and Safety Validation

- [ ] Verify no dropped/duplicated messages under stress.
- [ ] Verify ordering guarantees under each supported mode.
- [ ] Verify cancellation/shutdown behavior remains correct.
- [ ] Verify no deadlock/livelock under adversarial timing tests.

#### E. Benchmark Expansion

- [ ] Add dedicated channel microbench suite: SPSC, MPSC, MPMC, ping-pong, select-heavy.
- [ ] Capture p50/p95/p99 and variance for all channel microbenches.
- [ ] Run full suite and isolate channel contribution changes.

### Validation and Tests

- [ ] Concurrency stress tests (long-duration, randomized timing).
- [ ] Deterministic replay tests for known race-sensitive scenarios.
- [ ] Regression tests for existing channel semantics.

### Risks and Mitigations

- [ ] Risk: fast path introduces rare race. Mitigation: stress + sanitizers + fallback switch.
- [ ] Risk: spin policy hurts CPU efficiency. Mitigation: configurable thresholds + profile-based tuning.
- [ ] Risk: optimization favors one topology only. Mitigation: benchmark matrix across multiple topologies.

### Exit Criteria and Required Artifacts

- [ ] `channel_pingpong` mean runtime reduced by at least `3x` from current baseline.
- [ ] p95/p99 stability improves for channel microbenchmarks.
- [ ] Concurrency correctness tests remain green with no flaky regressions.
- [ ] Phase report includes latency breakdown and contention counter deltas.

---

## Phase 4 - String and JSON Conversion Efficiency (P2/P3)

### Primary Outcome

Reduce allocation-heavy conversion overhead in string/number and JSON helper paths.

### Scope

- In scope: parse/stringify optimization, temporary buffer reuse, JSON minify/validate hot paths.
- Out of scope: large feature additions to JSON subsystem.

### Detailed Implementation Checklist

#### A. Numeric Conversion Fast Paths

- [ ] Implement fast integer parse routine for common decimal patterns.
- [ ] Implement fast integer stringify routine with minimal allocation.
- [ ] Add checked overflow handling paths without penalizing common case.
- [ ] Add tiny-value fast path (single-digit and short-decimal cases).

#### B. Allocation and Buffer Reuse

- [ ] Introduce reusable scratch buffers where thread-safe and lifetime-safe.
- [ ] Minimize transient heap allocations in conversion loops.
- [ ] Verify no stale buffer aliasing bugs.
- [ ] Add counters for conversion allocations per operation.

#### C. JSON Utility Optimization

- [ ] Add `already_minified` fast path to minifier.
- [ ] Reduce branch-heavy checks in validation path.
- [ ] Optimize common object shape validation path (without changing correctness semantics).
- [ ] Ensure UTF-8 and escape handling semantics are preserved.

#### D. Benchmarks and Profiling

- [ ] Add dedicated microbenchmarks for parse, stringify, minify, validate.
- [ ] Profile call stacks before and after optimizations.
- [ ] Record allocation count deltas and RSS changes.
- [ ] Validate cross-language parity outputs remain unchanged.

### Validation and Tests

- [ ] Unit tests for edge numeric formats and invalid inputs.
- [ ] Property tests for parse/stringify round-trip stability.
- [ ] JSON correctness corpus tests (valid and invalid payloads).

### Risks and Mitigations

- [ ] Risk: fast paths break edge-case correctness. Mitigation: corpus and property testing.
- [ ] Risk: buffer reuse causes hidden data corruption. Mitigation: strict ownership tests and thread-safety review.

### Exit Criteria and Required Artifacts

- [ ] `string_concat_checksum` improves by at least `40%`.
- [ ] `json_roundtrip` improves by at least `25%`.
- [ ] RSS does not regress materially in conversion-heavy loops.
- [ ] Phase report includes before/after profiles and allocation metrics.

---

## Phase 5 - Compiler and Codegen Throughput (P4)

### Primary Outcome

Improve compile latency and throughput while preserving deterministic outputs and diagnostics quality.

### Scope

- In scope: phase timing instrumentation, allocation reduction in compiler pipeline, caching of immutable metadata.
- Out of scope: semantic changes to language behavior.

### Detailed Implementation Checklist

#### A. Measurement and Visibility

- [ ] Add phase timing spans (parse, resolve, typecheck, lower, codegen, link).
- [ ] Export compiler timing report in machine-readable format.
- [ ] Add CI summary for compile phase hotspots.

#### B. Front-End Throughput Improvements

- [ ] Reduce redundant allocations in parser and AST construction paths.
- [ ] Intern frequently repeated symbols/strings where safe.
- [ ] Reuse immutable metadata objects across compilation units.

#### C. Middle-End and Lowering Improvements

- [ ] Audit and remove repeated traversals where single-pass alternatives are possible.
- [ ] Reduce temporary structure churn in lowering.
- [ ] Cache stable analysis artifacts safely.

#### D. Backend and Link Steps

- [ ] Audit codegen staging overhead and remove duplicate serialization passes.
- [ ] Evaluate link invocation parameters for throughput without changing deterministic output.
- [ ] Add compile throughput guardrail benchmark for representative benchmark cases.

#### E. Determinism and Diagnostic Stability

- [ ] Add test asserting output artifact determinism across repeated builds.
- [ ] Add test asserting stable diagnostic ordering and messaging.
- [ ] Ensure timing instrumentation does not alter behavior or output content.

### Validation and Tests

- [ ] Compiler benchmark suite for clean builds on benchmark corpus.
- [ ] Determinism tests across repeated runs.
- [ ] Diagnostics snapshot tests.

### Risks and Mitigations

- [ ] Risk: throughput optimizations alter deterministic behavior. Mitigation: deterministic artifact checks in CI.
- [ ] Risk: aggressive caching introduces stale metadata bugs. Mitigation: cache invalidation tests and safe scoping.

### Exit Criteria and Required Artifacts

- [ ] Mean VibeLang compile latency improves by at least `20%` on benchmark suite.
- [ ] Determinism and diagnostics tests remain fully green.
- [ ] Phase report includes phase timing waterfall before/after.

---

## Phase 6 - Stabilization, Regression Gates, and Publication

### Primary Outcome

Convert optimization gains into durable engineering controls and transparent published evidence.

### Scope

- In scope: CI gates, thresholds, regression triage workflow, publication templates, refresh policy.
- Out of scope: major new runtime algorithm projects.

### Detailed Implementation Checklist

#### A. CI Performance Gates

- [ ] Add configurable per-case runtime budget thresholds.
- [ ] Add geomean budget thresholds for C/Rust/Go comparisons.
- [ ] Add fail/warn/severity policy levels.
- [ ] Add "required rerun count" logic for noisy concurrency regressions.
- [ ] Ensure CI output points to failing metric and baseline comparison artifact.

#### B. Regression Triage and Incident Workflow

- [ ] Create performance regression issue template.
- [ ] Define severity classes (critical, high, medium, low).
- [ ] Define owner and SLA for each severity class.
- [ ] Define rollback decision matrix when performance gates fail.
- [ ] Document emergency bypass policy and approval requirements.

#### C. Publication and Reporting

- [ ] Publish before/after optimization reports with explicit methodology.
- [ ] Include fairness notes and confidence caveats in published summaries.
- [ ] Archive raw JSON and generated markdown artifacts.
- [ ] Add changelog entry linking optimization commits to benchmark outcomes.

#### D. Ongoing Benchmark Governance

- [ ] Define quarterly benchmark refresh process (toolchain bumps and host refresh).
- [ ] Define annual benchmark case review to prevent stale or gameable workloads.
- [ ] Define periodic verification run on native Linux authoritative host.
- [ ] Define protocol for adding/removing benchmark cases with compatibility review.

### Validation and Tests

- [ ] Dry-run CI gates against historical artifacts.
- [ ] Simulate known regressions and verify correct gate behavior.
- [ ] Verify publication checklist completeness with one end-to-end rehearsal.

### Risks and Mitigations

- [ ] Risk: overly tight gates block valid changes. Mitigation: warn mode rollout then strict mode.
- [ ] Risk: noisy cases cause false positives. Mitigation: rerun policy and robust metrics.

### Exit Criteria and Required Artifacts

- [ ] CI automatically catches key runtime regressions with actionable output.
- [ ] Regression triage workflow is documented and trialed.
- [ ] Published optimization changelog includes reproducible evidence links.

---

## Rolling KPI Targets (Updated Baseline -> Target)

### Geomean Ratios

- [ ] vs C: `2.623` -> `< 1.90` (intermediate) -> `< 1.50` (longer-term)
- [ ] vs Rust: `1.909` -> `< 1.30` (intermediate) -> `< 1.15` (longer-term)
- [ ] vs Go: `2.752` -> `< 1.70` (intermediate) -> `< 1.20` (longer-term)

### Hotspot Case Ratios

- [ ] `hashmap_frequency` ratio vs C: `146.571` -> `< 15` (intermediate) -> `< 6` (longer-term)
- [ ] `channel_pingpong` ratio vs Go: `182.018` -> `< 25` (intermediate) -> `< 10` (longer-term)
- [ ] `string_concat_checksum` ratio vs C: `3.828` -> `< 2.2` (intermediate) -> `< 1.5` (longer-term)

### Stability and Noise

- [ ] Non-concurrency case run-to-run drift under `+/-5%` on authoritative host.
- [ ] Concurrency case p95 and p99 variance trend improves quarter over quarter.

---

## Suggested Execution Waves (Practical Sequencing)

- [ ] Wave A: complete Phase 0 hardening and baseline lock.
- [ ] Wave B: run Phase 1 and Phase 3 in parallel with independent owners.
- [ ] Wave C: complete Phase 2 integration cleanup after map backend stabilizes.
- [ ] Wave D: complete Phase 4 conversion/JSON optimization.
- [ ] Wave E: complete Phase 5 compiler throughput lane.
- [ ] Wave F: complete Phase 6 gates/publication and switch to maintenance cadence.

---

Use this file as the canonical optimization implementation tracker. Update checkboxes only when linked evidence artifacts are available under `reports/benchmarks/cross_lang/`.
