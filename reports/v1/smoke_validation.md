# V1 Tightening Smoke Validation (Local Dry-Run)

Date: 2026-02-21

## Executed Commands

All commands were executed from workspace root.

- `cargo test -p vibe_cli --test phase2_native`
- `cargo test -p vibe_cli --test frontend_fixtures ownership_err_golden`
- `cargo test -p vibe_cli --test frontend_fixtures parse_err_golden`
- `cargo test -p vibe_cli --test frontend_fixtures type_ok_fixtures`
- `cargo test -p vibe_cli --test frontend_fixtures type_err_golden`
- `cargo test -p vibe_cli --test frontend_fixtures snapshots_container_ops_mir_is_deterministic`
- `cargo test -p vibe_cli --test cli_help_snapshots`
- `cargo test -p vibe_cli --test cli_version`
- local packaged-install simulation (archive extract + extracted `vibe --version` + extracted `vibe run` hello-world)
- `cargo test -p vibe_cli --test phase4_indexer`
- `cargo test -p vibe_cli --test phase7_validation`
- `cargo test -p vibe_cli --test phase7_concurrency`
- `cargo test -p vibe_cli --test phase7_v1_tightening`
- `cargo test -p vibe_cli --test contract_runtime_enforcement`
- `cargo test -p vibe_cli --test phase2_native run_enforces_native_require_contracts`
- `cargo test -p vibe_cli --test phase2_native produced_binary_enforces_native_ensure_contracts`
- `cargo test -p vibe_cli --test phase2_native run_locked_requires_lockfile_when_manifest_exists`
- `cargo test -p vibe_cli --test phase2_native run_locked_succeeds_with_lockfile_when_manifest_exists`
- `cargo test -p vibe_cli --test phase2_native test_locked_requires_lockfile_when_manifest_exists`
- `cargo test -p vibe_cli --test phase2_native test_locked_succeeds_with_lockfile_when_manifest_exists`
- `cargo test -p vibe_cli --test phase7_v1_tightening phase7_algorithmic_recursion_samples_run_expected_outputs`
- `cargo test -p vibe_cli --test phase7_v1_tightening phase7_memory_heap_pressure_smoke_is_bounded`
- `cargo test -p vibe_cli --test phase7_v1_tightening phase7_gc_observable_smoke_is_default_lane`
- `cargo test -p vibe_cli --test phase7_v1_tightening phase7_memory_valgrind_leak_check_default_lane`
- `cargo test -p vibe_cli --test phase7_v1_tightening phase7_ownership_sendability_smokes_cover_positive_and_negative_paths`
- `python3 tooling/metrics/collect_phase6_metrics.py`
- `python3 tooling/metrics/validate_phase7_coverage_matrix.py`
- `python3 tooling/metrics/validate_phase6_metrics.py`
- `python3 tooling/metrics/validate_v1_quality_budgets.py`

## Result Summary

- Determinism-related integration tests: PASS
- Concurrency and bounded stress tests: PASS
- Algorithmic recursion smokes (Fibonacci/Factorial): PASS
- Ownership/sendability safety smokes: PASS
- Heap-pressure bounded smoke: PASS
- Dynamic container conformance smokes (`Str`/`List`/`Map`): PASS
- Native contract runtime enforcement suites: PASS
- CLI help/version regression smokes: PASS
- Local no-Cargo packaged install simulation (Linux layout): PASS
- Coverage/budget validators (clean/no-op/incremental + memory lanes): PASS

## Memory + GC Default Lanes

- Valgrind leak lane (`phase7_memory_valgrind_leak_check_default_lane`): PASS
- GC-observable lane (`phase7_gc_observable_smoke_is_default_lane`): PASS

## Notes

- Native execution-path contract enforcement is now wired as a blocking lane and tracked as
  closed in `reports/v1/readiness_dashboard.md`.
- Dynamic container closeout evidence is published in
  `reports/v1/dynamic_containers_conformance.md`.
- Independent install and distribution trust wiring evidence is published in
  `reports/v1/install_independence.md` and `reports/v1/distribution_readiness.md`.
- Consolidated Phase 8 local workflow-equivalent evidence and artifact links are
  published in `reports/v1/phase8_ci_evidence.md`.
- Final Phase 8 compliance audit summary is published in
  `reports/v1/phase8_closeout_summary.md`.
