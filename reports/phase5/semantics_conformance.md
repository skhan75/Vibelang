# Phase 5 Semantics Conformance Report

Date: 2026-02-17

## Scope

This report tracks native backend conformance evidence for Phase 5 control-flow and concurrency semantics from:

- `docs/spec/semantics.md`
- `compiler/tests/fixtures/build/*`
- `crates/vibe_cli/tests/phase2_native.rs`

## Executed Test Evidence

- Command: `cargo test -p vibe_cli --test phase2_native`
- Result: pass (`16 passed; 0 failed`)

## Conformance Matrix

| Semantic area | Evidence fixture/test | Status |
| --- | --- | --- |
| `while` native execution | `build/while_loop.vibe` + `while_loop_fixture_runs` | pass |
| `repeat` native execution | `build/repeat_loop.vibe` + `repeat_loop_fixture_runs` | pass |
| `go` detached task behavior | `build/concurrency_go_select.vibe` + `concurrency_go_select_fixture_runs` | pass |
| `select` receive + `after` | `build/select_after_timeout.vibe` + `select_after_timeout_fixture_runs` | pass |
| `select` receive + `default` | `build/select_default.vibe` + `select_default_fixture_runs` | pass |
| `select` `closed` case | `build/select_closed.vibe` + `select_closed_fixture_runs` | pass |
| multi-case receive dispatch | `build/select_multi_receive.vibe` + `select_multi_receive_fixture_runs` | pass |
| stable unsupported-form diagnostics | `build_err/member_access_unsupported.vibe` + `unsupported_member_access_has_stable_codegen_diagnostic` | pass |
| contract runtime policy (dev/test default) | `build/contract_runtime_require.vibe` + `vibe_test_enforces_contract_runtime_checks_by_default` | pass |
| contract runtime policy override | env `VIBE_CONTRACT_CHECKS=off` + `vibe_test_can_disable_contract_runtime_checks_with_env_override` | pass |

## Notes

- Native backend now includes `go` spawn ABI and multi-case `select` lowering with rotating start cursor.
- Unsupported backend forms are surfaced with stable `E340x` diagnostics instead of generic messages.
- Contract runtime checks are enforced in deterministic test execution path by default for non-release profiles, with explicit environment override support.
