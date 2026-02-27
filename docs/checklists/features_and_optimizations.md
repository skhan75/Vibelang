# VibeLang Features and Optimizations Checklist

Last updated: 2026-02-27

## Purpose

This is the canonical implementation checklist for feature gaps, limitations, and optimization items needed for VibeLang to reach mature production parity.

## Semantic-First Example Policy

- Examples must represent intended VibeLang semantics and ergonomics.
- Do not rewrite examples into workaround form only to pass current runtime limitations.
- If an example fails due to language/runtime/tooling limitations, keep the semantic example and track the gap here with an actionable fix item.
- Every failing example should map to at least one checklist item ID.

## Current Snapshot

- Example corpus: `78` programs under `examples/`
- Static status: all examples pass `vibe check`
- Runtime status (source-built CLI sweep): `73` pass / `5` fail
- Non-entry helper module files now fail with explicit entrypoint diagnostics (expected):
  - `examples/08_modules_packages/project_math/demo/math.yb`
  - `examples/08_modules_packages/project_pipeline/app/parser.yb`
  - `examples/08_modules_packages/project_pipeline/app/formatter.yb`
- Intentional failure demos (keep failing): `examples/10_contracts_intent/68_runtime_require_failure_demo.yb`, `examples/10_contracts_intent/69_runtime_ensure_failure_demo.yb`
- Release gate status: GA gates closed in `reports/v1/readiness_dashboard.md`
- Benchmark strict-publication status: blocked by items in section **B**

---

## A) Runtime and Codegen Parity Gaps (Examples Execution)

### A-01 (P0) `Str.len()` runtime parity
- [x] Implement stable lowering/runtime dispatch for string length.
- **Symptoms**: `panic: container len called on unsupported container`
- **Impacted examples**:
  - `examples/02_strings_numbers/11_string_len_compare.yb`
  - `examples/02_strings_numbers/12_string_build_loop.yb`
  - `examples/05_graphs_recursion_patterns/36_palindrome_check.yb`
  - `examples/08_modules_packages/project_pipeline/app/main.yb` (via `parser.yb`)
- **Likely subsystem**: codegen + runtime dispatch
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - Add integration tests for `s.len()` in loops and helper functions

### A-02 (P0) List method dispatch parity (`.get` / `.set`)
- [x] Fix list receiver dispatch so list methods never route to map/string-key paths.
- **Symptoms**: `panic: container get(Str)...`, `panic: container set(Str, Int)...`
- **Impacted examples**:
  - `examples/03_data_structures/16_list_append_get_set.yb`
  - `examples/03_data_structures/19_queue_with_list.yb`
  - `examples/04_algorithms/23_linear_search.yb`
  - `examples/04_algorithms/24_binary_search.yb`
  - `examples/04_algorithms/25_bubble_sort.yb`
  - `examples/04_algorithms/26_selection_sort.yb`
  - `examples/04_algorithms/27_prefix_sum.yb`
  - `examples/05_graphs_recursion_patterns/32_graph_bfs_small.yb`
  - `examples/05_graphs_recursion_patterns/34_tree_sum_recursive.yb`
  - `examples/09_agentic_patterns/47_agentic_guardrail_pipeline.yb`
- **Likely subsystem**: runtime method dispatch + codegen receiver typing
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - Add list method conformance tests (index patterns, loops, recursion)

### A-03 (P0) `Map<Int, Int>` method parity
- [x] Implement correct key-type dispatch for integer-key maps (`get/set/contains/remove`).
- **Symptoms**: int-key map calls route through str-key runtime and panic.
- **Impacted examples**:
  - `examples/03_data_structures/21_map_int_int_basics.yb`
  - `examples/05_graphs_recursion_patterns/33_graph_dfs_small.yb`
- **Acceptance**:
  - Both examples pass `vibe run`
  - Add explicit `Map<Int, Int>` runtime conformance tests for all methods

### A-04 (P0) Missing builtin lowering (`max`, global `len`)
- [x] Implement/restore codegen lowering for builtins referenced in language surface.
- **Symptoms**: `E3403: unknown call target`
- **Impacted examples**:
  - `examples/02_strings_numbers/13_int_arithmetic_min_max.yb` (`max`)
  - `examples/05_graphs_recursion_patterns/35_tree_depth_recursive.yb` (`max`)
  - `examples/10_contracts_intent/66_list_transform_contracts.yb` (`len`)
  - `examples/10_contracts_intent/67_public_api_style_contracts.yb` (`len`)
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - Add builtins smoke tests for `min/max/len` under `vibe run` and `vibe test`

### A-05 (P0) `.sort_desc()` native backend support
- [x] Implement list sort lowering in native backend for supported list types.
- **Symptoms**: `E3404: member call .sort_desc() is not supported in v0.1 native backend`
- **Impacted examples**:
  - `examples/03_data_structures/17_list_sort_take.yb`
  - `examples/10_contracts_intent/66_list_transform_contracts.yb`
  - `examples/10_contracts_intent/67_public_api_style_contracts.yb`
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - Deterministic ordering tests added for sorted output

### A-06 (P0) Float codegen/runtime stability
- [x] Fix float value typing/lowering and verifier failures.
- **Symptoms**: verifier errors / backend panic on float examples.
- **Impacted examples**:
  - `examples/02_strings_numbers/14_float_basics.yb`
  - `examples/02_strings_numbers/15_float_comparison.yb`
- **Acceptance**:
  - Both examples pass `vibe run`
  - Add float arithmetic/comparison integration tests

### A-07 (P0) Contract example-runner parity
- [x] Align `vibe test` example evaluator with executable language surface.
- **Symptoms**: contract/example preflight rejects methods used in regular code.
- **Impacted examples**:
  - `examples/10_contracts_intent/60_effect_alloc_mut_state.yb`
- **Acceptance**:
  - Contract examples using list/map methods execute under `vibe test`
  - If a subset is intentionally restricted, enforce with explicit diagnostics and docs

### A-08 (P1) Module helper-file run ergonomics
- [x] Improve CLI error for non-entry module execution.
- **Symptoms**: link error `undefined reference to main` when running helper module files.
- **Impacted examples**:
  - `examples/08_modules_packages/project_math/demo/math.yb`
  - `examples/08_modules_packages/project_pipeline/app/parser.yb`
  - `examples/08_modules_packages/project_pipeline/app/formatter.yb`
- **Acceptance**:
  - User-friendly diagnostic points to module entry file
  - Docs include explicit entrypoint run guidance

---

## B) Benchmark Publication and Performance Blockers

### B-01 (P1) Noncanonical benchmark adapters (strict publication gate)
- [ ] Canonicalize remaining adapters:
  - `edigits`
  - `http-server`
  - `json-serde`
  - `secp256k1`
- **Evidence**:
  - `reports/benchmarks/third_party/analysis/gaps_optimization_blocker_checklist.md`
  - `reports/benchmarks/third_party/full/summary.md`
- **Acceptance**:
  - Parity validator passes in publication mode

### B-02 (P1) Missing strict lane (`zig`)
- [ ] Restore required runtime/compile lane availability for `zig` in strict benchmark mode.
- Runtime/compile lanes for `rust` and `swift` are available in current source-driven no-docker sweeps.
- **Evidence**: `reports/benchmarks/third_party/full/summary.md`
- **Acceptance**:
  - No missing required lanes in strict publication report

### B-03 (P1) Zig local compatibility readiness
- [ ] Ensure local Zig toolchain/version is compatible with PLB-CI source set used in strict checks.
- **Evidence**:
  - `reports/benchmarks/third_party/full/results.json`
  - `reports/benchmarks/third_party/analysis/gaps_optimization_blocker_checklist.md`
- **Acceptance**:
  - Local `zig` lane builds and produces runtime/compile artifacts without compatibility failures

### B-04 (P1) Docker strict run stability
- [ ] Stabilize Docker-backed strict run path.
- **Evidence**: `reports/benchmarks/third_party/analysis/gaps_optimization_blocker_checklist.md`
- **Acceptance**:
  - Consecutive strict docker-backed runs complete without daemon failures

### B-05 (P2) Performance optimization backlog
- [ ] Close runtime gap vs Kotlin and compile-cold gap vs C/C++/Go after parity fixes.
- **Evidence**: `reports/benchmarks/third_party/full/summary.md`
- **Acceptance**:
  - Defined performance SLOs met in strict mode and tracked in trend reports

---

## C) Core Language Surface Gaps (Spec vs Executable Surface)

### C-00 (P0) Data-modeling direction lock (types-first)
- [x] Lock and publish the canonical data-modeling direction:
  - first-class nominal `type` declarations are the primary model for related mixed-type data
  - composition-first behavior reuse is the default
  - inheritance/class model remains explicitly gated by C-03 decision
  - structural "shape"-style modeling (if adopted) is boundary-focused, not a replacement for nominal types
- **Why**: avoid map-record drift and keep language semantics consistent with production expectations.
- **Acceptance**:
  - Direction is documented in spec docs and mirrored in examples policy
  - `examples/11_modeling_shapes/` notes are aligned to this direction

### C-01 (P0) User-defined type declarations (struct-like shapes)
- [x] Implement end-to-end executable support for `type` declarations with mixed field types (MVP: heap records, 8-byte slots).
- **Why**: Required for real struct/shape modeling parity (C++/Rust-like workflows).
- **Evidence**:
  - Spec syntax exists: `docs/spec/syntax.md` (`type` declaration section)
  - AST/parser declaration path remains function-centric:
    - `crates/vibe_ast/src/lib.rs`
    - `crates/vibe_parser/src/lib.rs`
- **Acceptance**:
  - Support mixed field types (`Int`, `Str`, `Bool`, `List<T>`, `Map<K,V>`, optional `T?`)
  - Typed field read/update with compile-time diagnostics on unknown/mismatched fields
  - Add runnable examples replacing map-record stand-ins in `examples/11_modeling_shapes/`

### C-01a (P0) Type construction and update ergonomics
- [x] Define and implement canonical construction/update ergonomics for `type` values (MVP: `Type { field: expr }`, `obj.field`, `obj.field = expr`).
- **Why**: production use needs concise, safe creation/update patterns (not map-style emulation).
- **Acceptance**:
  - One canonical constructor model documented (literal-style and/or constructor function)
  - Deterministic diagnostics for missing required fields and invalid field assignments
  - At least two end-to-end runnable examples (simple domain model + nested model)

### C-02 (P0) Enum and executable `match` support
- [x] Add enum syntax + value model + type checking + match lowering + exhaustiveness (MVP: no-payload enums, tag-based match).
- **Why**: Pattern matching parity and algebraic data modeling.
- **Evidence**:
  - `docs/spec/control_flow.md` and `docs/spec/type_system.md` reference match/exhaustiveness
  - Implementation status remains partial/deferred in spec coverage docs
- **Acceptance**:
  - Runnable enum/match examples and integration tests

### C-03 (P1) Inheritance/class model decision
- [ ] Decide and document whether inheritance is in-scope.
- **Options**:
  - Adopt inheritance and implement
  - Declare explicit non-goal and formalize composition-first model
- **Acceptance**:
  - Decision reflected in spec, checklist, and examples
  - If non-goal: add explicit migration guidance from inheritance patterns to composition patterns

### C-04 (P1) Trait/interface polymorphism
- [ ] Define and implement trait/interface MVP, or formally defer with migration guidance.
- **Evidence**: deferred notes in `docs/spec/type_system.md`

### C-04a (P1) Composition-first patterns as first-class guidance
- [ ] Publish normative examples for capability composition with nominal types.
- **Why**: if inheritance is deferred/non-goal, composition guidance must be explicit and runnable.
- **Acceptance**:
  - Add composition patterns using real `type` declarations (not map stand-ins)
  - Include at least one "policy + core model" example and one "pipeline composition" example

### C-05a (P1) Optional structural shape support (boundary payloads)
- [ ] Decide whether structural "shape" typing is needed for external payload boundaries.
- **Why**: users may want PHP/TS-like shape ergonomics for request/response/input schemas.
- **Acceptance**:
  - If adopted: specify syntax, typing rules, and relation to nominal `type`
  - If deferred/non-goal: document recommended nominal-type alternative and conversion patterns

### C-05 (P1) `mut` / `const` / optional ergonomics
- [ ] Close parser/typechecker/runtime gaps for explicit mutability and optional value semantics.
- **Why**: ergonomic parity and clear data-flow intent in public APIs.

### C-06 (P1) Generic container support beyond current freeze
- [ ] Expand executable support for `List<T>` / `Map<K,V>` combinations beyond narrow runtime freeze combinations.
- **Evidence**: deferred notes in `docs/spec/containers.md`

### C-07 (P2) Numeric width fidelity (`i32`, `u64`, `f64`) in executable surface
- [ ] Implement first-class width-aware numeric behavior and conversion checks.

---

## D) Example Program Quality Gates (Required Before “Production-Ready Examples” Claim)

### D-01 (P1) CI static check sweep
- [x] Add CI job: run `vibe check` on all examples.

### D-02 (P1) CI runtime sweep for runnable entries
- [x] Add CI job: run `vibe run` on all non-demo entry examples.

### D-03 (P1) Intentional-failure allowlist governance
- [x] Maintain explicit allowlist of intentional-failure demo examples.

### D-04 (P1) Checklist-ID linkage for failures
- [x] Require every failing example to reference a checklist ID from this file.

### D-05 (P1) Parity trend reporting
- [x] Publish periodic example parity report (pass/fail trend).

---

## E) Recommended Execution Order

1. **P0 runtime/codegen parity**: A-01..A-07
2. **P0 language data-modeling core**: C-00, C-01, C-01a, C-02
3. **P1 benchmark strict-publication blockers**: B-01..B-04
4. **P1 modeling decisions and ergonomics**: C-03, C-04, C-04a, C-05, C-05a, C-06
5. **P2 optimization and advanced surface**: B-05, C-07

This order ensures semantic examples can remain “truthful” while the implementation catches up without workaround drift.
