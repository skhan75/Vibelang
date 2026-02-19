# Phase 7.1 Sample Programs Catalog

Date: 2026-02-17

## Single-Thread Samples

| Sample | Fixture Path | Command | Expected Output |
| --- | --- | --- | --- |
| Hello World | `compiler/tests/fixtures/phase7/advanced/single_thread/single_thread__hello_world.yb` | `vibe run <file>` | `phase7-hello` |
| Calculator | `compiler/tests/fixtures/phase7/advanced/single_thread/single_thread__calculator.yb` | `vibe run <file>` | `calc-ok` |
| Pipeline Transform | `compiler/tests/fixtures/phase7/advanced/single_thread/single_thread__pipeline_transform.yb` | `vibe run <file>` | `pipe-ok` |
| State Machine | `compiler/tests/fixtures/phase7/advanced/single_thread/single_thread__state_machine.yb` | `vibe run <file>` | `state-ok` |
| Language Tour | `compiler/tests/fixtures/phase7/advanced/single_thread/single_thread__language_tour.yb` | `vibe run <file>` + `vibe test <file>` | `tour-ok` + examples pass |

## Multi-Thread / Concurrency Samples

| Sample | Fixture Path | Command | Expected Output |
| --- | --- | --- | --- |
| Worker Pool Pattern | `compiler/tests/fixtures/phase7/advanced/concurrency/concurrency__worker_pool.yb` | `vibe run <file>` | `worker-pool-ok` |
| Fan-In Pattern | `compiler/tests/fixtures/phase7/advanced/concurrency/concurrency__fan_in.yb` | `vibe run <file>` | `fan-in-ok` |
| Fan-Out Pattern | `compiler/tests/fixtures/phase7/advanced/concurrency/concurrency__fan_out.yb` | `vibe run <file>` | `fan-out-ok` |
| Timeout + Retry Pattern | `compiler/tests/fixtures/phase7/advanced/concurrency/concurrency__timeout_retry.yb` | `vibe run <file>` | `retry-attempt-1` then `retry-ok` |
| Bounded Stress Scenario | `compiler/tests/fixtures/phase7/stress/concurrency/concurrency__bounded_stress.yb` | `vibe run <file>` | `stress-ok` |

## Concurrency Misuse Diagnostics

| Fixture | Expected Diagnostic |
| --- | --- |
| `compiler/tests/fixtures/phase7/advanced/concurrency_err/concurrency_err__member_capture_in_go.yb` | `E3202` (go member capture alias risk) |
| `compiler/tests/fixtures/phase7/advanced/concurrency_err/concurrency_err__shared_member_write.yb` | `E3203` (shared mutable write in concurrent function) |

## Notes

- Run all sample checks via `cargo test -p vibe_cli --test phase7_validation` and `cargo test -p vibe_cli --test phase7_concurrency`.
- Determinism checks for outputs and build artifacts are part of the same test suites.
