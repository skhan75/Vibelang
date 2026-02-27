# VibeLang Example Coverage and Runtime Parity Checklist

This checklist is the canonical tracker for missing language/runtime/compiler features
that block full example execution parity.

## Current status snapshot

- Total examples: `70`
- `vibe check` status: all examples clean
- `vibe run` status: `41` pass, `29` fail
- Goal: all non-demo entry examples compile and run successfully with expected output

## Execution blockers discovered from full run sweep

- [ ] **R1: String length runtime parity (`Str.len`)**
  - Symptom: `panic: container len called on unsupported container`.
  - Affects:
    - `examples/02_strings_numbers/11_string_len_compare.yb`
    - `examples/02_strings_numbers/12_string_build_loop.yb`
    - `examples/05_graphs_recursion_patterns/36_palindrome_check.yb`
    - `examples/08_modules_packages/project_pipeline/app/main.yb`
  - Build/fix:
    - Ensure member `.len()` dispatch for `Str` always lowers to string runtime len path.
    - Add regression tests for both `s.len()` and nested string-len usage in loops.

- [ ] **R2: List method dispatch parity (`.get` / `.set`)**
  - Symptom: `panic: container get(Str)...` and `panic: container set(Str, Int)...` from list usage.
  - Affects:
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
  - Build/fix:
    - Correct receiver-type dispatch for list container methods in runtime/codegen.
    - Add list get/set roundtrip tests across loops and recursion.

- [ ] **R3: `Map<Int, Int>` runtime method parity**
  - Symptom: map int-key ops route through str-key path and panic.
  - Affects:
    - `examples/03_data_structures/21_map_int_int_basics.yb`
    - `examples/05_graphs_recursion_patterns/33_graph_dfs_small.yb` (`remove` path)
  - Build/fix:
    - Separate int-key and str-key runtime entry points for `set/get/contains/remove`.
    - Add explicit `Map<Int,Int>` conformance tests for all methods.

- [ ] **R4: Builtin/codegen surface gaps (`max`, global `len`)**
  - Symptom: `E3403: unknown call target`.
  - Affects:
    - `examples/02_strings_numbers/13_int_arithmetic_min_max.yb` (`max`)
    - `examples/05_graphs_recursion_patterns/35_tree_depth_recursive.yb` (`max`)
    - `examples/10_contracts_intent/66_list_transform_contracts.yb` (global `len`)
    - `examples/10_contracts_intent/67_public_api_style_contracts.yb` (global `len`)
  - Build/fix:
    - Implement or re-enable codegen lowering for missing global builtins.
    - Add `vibe run` tests for `min/max/len` call targets.

- [ ] **R5: List sorting codegen parity (`.sort_desc`)**
  - Symptom: `E3404: member call .sort_desc() is not supported in v0.1 native backend`.
  - Affects:
    - `examples/03_data_structures/17_list_sort_take.yb`
  - Build/fix:
    - Implement native backend lowering for `.sort_desc()` on supported list shapes.
    - Add deterministic ordering tests for sorted output.

- [ ] **R6: Float execution/codegen stability**
  - Symptom: verifier errors / backend panic on float examples.
  - Affects:
    - `examples/02_strings_numbers/14_float_basics.yb`
    - `examples/02_strings_numbers/15_float_comparison.yb`
  - Build/fix:
    - Fix float variable typing and arithmetic lowering in backend.
    - Add float comparison/arithmetic run tests in CLI integration suite.

- [ ] **R7: Contract example-runner surface mismatch**
  - Symptom: contract/example preflight rejects method usage in examples (`.append()` not supported in phase2 examples).
  - Affects:
    - `examples/10_contracts_intent/60_effect_alloc_mut_state.yb`
  - Build/fix:
    - Align `vibe test` example evaluator with language surface used by examples.
    - Or document/enforce restricted `@examples` subset with compiler diagnostics.

- [ ] **R8: Module helper file run ergonomics**
  - Symptom: running non-entry helper modules fails at link (`undefined reference to main`).
  - Affects:
    - `examples/08_modules_packages/project_math/demo/math.yb`
    - `examples/08_modules_packages/project_pipeline/app/parser.yb`
    - `examples/08_modules_packages/project_pipeline/app/formatter.yb`
  - Build/fix:
    - Improve CLI diagnostic for non-entry module execution.
    - Add explicit example-runner policy: run entry modules only for multi-file projects.

## Existing alternatives already demonstrated

- Shape modeling with today's stable surface:
  - `examples/11_modeling_shapes/71_shape_with_map_record.yb`
  - `examples/11_modeling_shapes/72_shape_contracts_and_validation.yb`
- Composition pattern as current alternative to inheritance:
  - `examples/11_modeling_shapes/73_composition_over_inheritance.yb`
- Async/await runnable references:
  - `examples/06_concurrency_async/42_async_thread_bridge.yb`
  - `examples/06_concurrency_async/43_async_await_pipeline.yb`
  - `examples/06_concurrency_async/44_async_await_parallel_join.yb`

## Expected-failure demo files (not blockers)

- `examples/10_contracts_intent/68_runtime_require_failure_demo.yb`
- `examples/10_contracts_intent/69_runtime_ensure_failure_demo.yb`

These are intentional contract-failure demonstrations and should stay failing under `vibe run`.

## Language-surface expansion (requested features, still pending)

- [ ] User-defined `type` declarations (struct-like domain models)
- [ ] Enums and executable `match` lowering
- [ ] Traits/interfaces (if in scope)
- [ ] Inheritance/class model decision (adopt vs explicit non-goal)
- [ ] Explicit `mut`/`const` bindings
- [ ] Optional ergonomics (`none`, `T?`) end-to-end
- [ ] Width-specific numeric types in executable surface (`i32/u64/f64`)

## Suggested priority order (IC8 lead gate)

- [ ] **P0**: R1, R2, R3 (core runtime container parity)
- [ ] **P0**: R4, R5, R6 (codegen/builtin parity for mainstream examples)
- [ ] **P0**: R7 (contract example-runner parity or explicit subset policy)
- [ ] **P1**: R8 (entrypoint diagnostics + runner ergonomics)
- [ ] **P1/P2**: language-surface expansion items above
