# VibeLang Examples (75 programs)

This directory contains runnable sample programs that cover VibeLang from basic syntax to advanced concurrency and modular projects.

## How to run

- From repo root:
  - `vibe run examples/01_basics/01_hello_world.yb`
- Static validation for all examples:
  - `rg --files examples -g '*.yb' | while read -r f; do vibe check "$f" || exit 1; done`
- For module-import projects, run the entry file:
  - `vibe run examples/08_modules_packages/project_math/demo/main.yb`
  - `vibe run examples/08_modules_packages/project_pipeline/app/main.yb`
- For contract/example execution:
  - `vibe test examples/10_contracts_intent/63_all_annotations_combo.yb`
- If a released binary is behind current language/runtime surface, use `vibe check` first and consult `examples/FEATURE_GAPS_CHECKLIST.md`.

## Coverage map

- Basics: `examples/01_basics/`
- Strings and numbers: `examples/02_strings_numbers/`
- Data structures: `examples/03_data_structures/`
- Algorithms: `examples/04_algorithms/`
- Graphs, recursion, patterns: `examples/05_graphs_recursion_patterns/`
- Concurrency and async: `examples/06_concurrency_async/`
- Stdlib IO/JSON/Regex/HTTP: `examples/07_stdlib_io_json_regex_http/`
- Modules and imports: `examples/08_modules_packages/`
- Agentic workflow patterns: `examples/09_agentic_patterns/`
- Intent and contracts: `examples/10_contracts_intent/`
- Shapes and composition patterns: `examples/11_modeling_shapes/`

## Example index

1. `01_basics/01_hello_world.yb`
2. `01_basics/02_variables_and_reassignment.yb`
3. `01_basics/03_comments_and_readability.yb`
4. `01_basics/04_functions_parameters_return.yb`
5. `01_basics/05_if_else.yb`
6. `01_basics/06_while_loop_counter.yb`
7. `01_basics/07_for_loop_over_list.yb`
8. `01_basics/08_break_continue_loop.yb`
9. `02_strings_numbers/09_string_concat.yb`
10. `02_strings_numbers/10_string_slice_substring.yb`
11. `02_strings_numbers/11_string_len_compare.yb`
12. `02_strings_numbers/12_string_build_loop.yb`
13. `02_strings_numbers/13_int_arithmetic_min_max.yb`
14. `02_strings_numbers/14_float_basics.yb`
15. `02_strings_numbers/15_float_comparison.yb`
16. `03_data_structures/16_list_append_get_set.yb`
17. `03_data_structures/17_list_sort_take.yb`
18. `03_data_structures/18_stack_with_list.yb`
19. `03_data_structures/19_queue_with_list.yb`
20. `03_data_structures/20_map_str_int_basics.yb`
21. `03_data_structures/21_map_int_int_basics.yb`
22. `03_data_structures/22_map_contains_remove.yb`
23. `04_algorithms/23_linear_search.yb`
24. `04_algorithms/24_binary_search.yb`
25. `04_algorithms/25_bubble_sort.yb`
26. `04_algorithms/26_selection_sort.yb`
27. `04_algorithms/27_prefix_sum.yb`
28. `04_algorithms/28_gcd_euclid.yb`
29. `04_algorithms/29_fibonacci_iterative.yb`
30. `04_algorithms/30_fibonacci_recursive.yb`
31. `04_algorithms/31_factorial_recursive.yb`
32. `05_graphs_recursion_patterns/32_graph_bfs_small.yb`
33. `05_graphs_recursion_patterns/33_graph_dfs_small.yb`
34. `05_graphs_recursion_patterns/34_tree_sum_recursive.yb`
35. `05_graphs_recursion_patterns/35_tree_depth_recursive.yb`
36. `05_graphs_recursion_patterns/36_palindrome_check.yb`
37. `06_concurrency_async/37_go_channel_basic.yb`
38. `06_concurrency_async/38_worker_pool_minimal.yb`
39. `06_concurrency_async/39_fan_in_pattern.yb`
40. `06_concurrency_async/40_fan_out_pattern.yb`
41. `06_concurrency_async/41_select_timeout_retry.yb`
42. `06_concurrency_async/42_async_thread_bridge.yb`
43. `07_stdlib_io_json_regex_http/43_fs_read_write_exists.yb`
44. `07_stdlib_io_json_regex_http/44_path_and_time_helpers.yb`
45. `07_stdlib_io_json_regex_http/45_json_basics.yb`
46. `07_stdlib_io_json_regex_http/46_regex_http_helpers.yb`
47. `09_agentic_patterns/47_agentic_guardrail_pipeline.yb`
48. `09_agentic_patterns/48_agentic_retry_budget.yb`
49. `08_modules_packages/project_math/demo/main.yb`
50. `08_modules_packages/project_math/demo/math.yb`
51. `08_modules_packages/project_pipeline/app/main.yb`
52. `08_modules_packages/project_pipeline/app/parser.yb`
53. `08_modules_packages/project_pipeline/app/formatter.yb`
54. `10_contracts_intent/54_intent_minimal.yb`
55. `10_contracts_intent/55_examples_table.yb`
56. `10_contracts_intent/56_require_precondition_guard.yb`
57. `10_contracts_intent/57_ensure_result_placeholder.yb`
58. `10_contracts_intent/58_ensure_old_snapshot.yb`
59. `10_contracts_intent/59_multiple_ensures.yb`
60. `10_contracts_intent/60_effect_alloc_mut_state.yb`
61. `10_contracts_intent/61_effect_io_nondet.yb`
62. `10_contracts_intent/62_transitive_effects_io.yb`
63. `10_contracts_intent/63_all_annotations_combo.yb`
64. `10_contracts_intent/64_agentic_planner_contract.yb`
65. `10_contracts_intent/65_agentic_executor_guardrails.yb`
66. `10_contracts_intent/66_list_transform_contracts.yb`
67. `10_contracts_intent/67_public_api_style_contracts.yb`
68. `10_contracts_intent/68_runtime_require_failure_demo.yb`
69. `10_contracts_intent/69_runtime_ensure_failure_demo.yb`
70. `10_contracts_intent/70_concurrency_effect_contracts.yb`
71. `11_modeling_shapes/71_shape_with_map_record.yb`
72. `11_modeling_shapes/72_shape_contracts_and_validation.yb`
73. `11_modeling_shapes/73_composition_over_inheritance.yb`
74. `06_concurrency_async/43_async_await_pipeline.yb`
75. `06_concurrency_async/44_async_await_parallel_join.yb`

## Feature gap tracker

- See `examples/FEATURE_GAPS_CHECKLIST.md` for unsupported or partially implemented language surfaces that affect certain requested example styles.
