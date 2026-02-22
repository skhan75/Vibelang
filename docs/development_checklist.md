# VibeLang Development Checklist (v1 Roadmap)

This checklist is the working tracker for building VibeLang from current docs into a production-ready language ecosystem.

Use rules:

- Mark a task complete only when code/tests/docs are merged and runnable.
- Keep checkboxes small and verifiable.
- If scope changes, update this checklist before implementation.

### Completion Evidence Rule (All Phases)

- Every checked item in a `Phase X Exit Criteria` section must cite at least one reproducible evidence artifact (CI workflow/job, test path, snapshot, or report).
- If implementation exists but evidence artifacts are missing, keep the corresponding exit criterion unchecked until evidence is linked.
- Preferred evidence format in checklist lines: `workflow <path> job <name>`, `test <path>`, `report <path>`.

## Critical Design Guardrails (Non-Negotiable)

### Determinism First

- [ ] Reproducible build mode (`--locked`, pinned toolchain, normalized artifacts)
- [ ] Determinism tests: same source + same toolchain => bit-identical output
- [x] AI sidecar proven non-blocking for parse/type/codegen/link paths (evidence: workflow `.github/workflows/phase5-ai-sidecar.yml` job `non_blocking_compile`, report `reports/phase5/workflow_impact.md`)
- [x] Deterministic diagnostics ordering in compiler output

### Safety Defaults

- [x] Data-race safety strategy documented and enforced in language/runtime behavior
- [ ] Memory safety defaults documented for user code paths
- [x] Concurrency primitives (`go`, `chan`, `select`) validated with stress tests
- [ ] Contract checks (`@require/@ensure`) active in dev/test profiles by default

### Escape Hatches (Isolated and Auditable)

- [ ] Define unsafe/low-level escape hatch syntax and scope boundaries
- [ ] Require explicit annotation/review path for unsafe blocks
- [ ] Emit audit report listing all unsafe blocks per build

### Transparent Performance Model

- [ ] Allocation visibility available in diagnostics/profile outputs
- [x] Effect declarations (`@effect`) checked against observed behavior
- [ ] Benchmark suite publishes CPU/memory/latency metrics per release
- [ ] Docs explain cost model for copies, allocations, and concurrency operations

### Fast Compile Times

- [ ] Baseline compile benchmarks for clean/no-op/incremental scenarios
- [ ] Incremental cache hit-rate telemetry in CI and local runs
- [ ] Regression thresholds configured for compile latency

### Immediate Guardrail Gaps (Audit: 2026-02-17)

- [ ] **P0:** Implement true reproducible build mode in CLI (`--locked`) and CI usage (`cargo ... --locked`) to match documented policy (evidence gap: `tooling/build_system.md` documents `vibe build --offline --locked`, but `crates/vibe_cli/src/main.rs` `parse_build_like_args` rejects `--locked`)
- [ ] **P0:** Enforce `@require/@ensure` in native dev/test execution path (not only example-runner path) and add failure-mode tests for `vibe run`/native binaries (evidence gap: `crates/vibe_cli/src/example_runner.rs` enforces contracts, while `crates/vibe_mir/src/lib.rs` has no contract lowering)
- [ ] **P0:** Fix sendability safety mismatch for unknown types in concurrent calls and align implementation with spec (evidence gap: `docs/spec/ownership_sendability.md` says unknown values are not sendable, but `crates/vibe_types/src/ownership.rs` currently treats `TypeKind::Unknown` as sendable)
- [x] **P0:** Implement native dynamic data-structure support for `Str`/`List`/`Map` construction and mutation paths, and remove release-critical `E3401`/`E3402` fallbacks from official sample coverage (sequence: execute after `7.3.e Compiler Self-Host Readiness` gate) (evidence: `reports/v1/dynamic_containers_conformance.md`, workflow `.github/workflows/v1-release-gates.yml` job `dynamic_containers_gate`)
- [ ] **P1:** Normalize reproducibility metadata and artifact paths so outputs are machine/path-stable (evidence gap: `crates/vibe_cli/src/main.rs` `write_debug_map` writes `source_path.display()` directly)
- [ ] **P1:** Pin toolchain to an exact Rust version (not moving `stable`) and add release evidence for toolchain hash + lockfile state
- [ ] **P1:** Expose incremental cache hit/miss telemetry in CLI/CI and gate minimum hit-rate in regression checks (evidence gap: telemetry fields exist in `crates/vibe_indexer/src/incremental.rs` but are not surfaced by `vibe index --stats`)
- [ ] **P1:** Replace compile timing smoke (`cargo run` wall time) with direct `vibe` binary clean/no-op/incremental benchmarks and enforce numeric thresholds (evidence gap: `tooling/metrics/collect_phase6_metrics.py` measures `cargo run` time and `tooling/metrics/validate_phase6_metrics.py` only validates `> 0`)

---

## Phase 1: Language Spec + Parser + Type Checker + Simple IR

Goal: lock core language behavior and ship a usable frontend that can type-check programs and emit simple IR.

### 1.1 Language Specification

- [x] Syntax spec draft (`docs/spec/syntax.md`)
- [x] Semantics spec draft (`docs/spec/semantics.md`)
- [x] Contracts spec draft (`docs/spec/contracts.md`)
- [x] Intent spec draft (`docs/spec/intents.md`)
- [x] Examples spec draft (`docs/spec/examples.md`)
- [x] Resolve open syntax ambiguities (block style, inference boundaries, effect vocabulary stability)
- [x] Freeze v0.1 grammar for parser implementation

### 1.2 Frontend Implementation

- [x] Create compiler crate/module structure for lexer/parser/AST
- [x] Implement lexer with token spans and recovery
- [x] Implement parser with contract annotation nodes
- [x] Implement AST validation and binder/name resolution
- [x] Implement type checker with local inference and `Result` propagation
- [x] Emit typed HIR (simple IR) for downstream phases

### 1.3 Diagnostics and Tests

- [x] Frontend test plan drafted (`compiler/tests/frontend_cases.md`)
- [x] Snapshot/golden diagnostics harness in CI
- [x] Parse recovery tests (multi-error reporting)
- [x] Type error suite (mismatch, unknown symbol, invalid contract expression)
- [x] Contract position and determinism rule tests

### 1.4 Engineering-Ready Gap Closure

- [x] Create Rust workspace/bootstrap for frontend crates and command wiring (`vibe check`, `vibe ast`, `vibe hir`)
- [x] Add formal grammar freeze artifact (`docs/spec/grammar_v0_1.ebnf`) and treat it as parser source of truth
- [x] Add resolved decisions appendix for Phase 1 syntax/semantics ambiguities
- [x] Add AST schema notes and HIR schema notes with a verifier checklist
- [x] Define deterministic diagnostics code ranges (`E1xxx`, `E2xxx`, `E3xxx`) and ordering guarantees
- [x] Add fixture-backed snapshot outputs for diagnostics, AST, and HIR
- [x] Add lexer/parser fuzz smoke tests (time-bounded) for panic resistance
- [x] Add determinism tests for repeated `check` and `hir` output on same inputs

### Phase 1 Exit Criteria

- [x] `vibe check` validates sample files end-to-end (test `crates/vibe_cli/tests/frontend_fixtures.rs`, workflow `.github/workflows/phase1-frontend.yml` job `tests`)
- [x] Typed HIR generated for core language constructs (test `crates/vibe_cli/tests/frontend_fixtures.rs` case `snapshots_ast_hir_and_diag`)
- [x] Frontend test suite stable in CI (workflow `.github/workflows/phase1-frontend.yml` jobs `fmt_lint`, `tests`)
- [x] Deterministic diagnostics and HIR output confirmed by repeat-run tests (workflow `.github/workflows/phase1-frontend.yml` job `determinism_smoke`)

---

## Phase 2: Native Backend + Small Stdlib + CLI Tooling

Goal: compile typed IR into native binaries with a minimal standard library and stable CLI workflow.

### 2.1 Codegen and Linking

- [x] Codegen strategy documented (`compiler/codegen/README.md`)
- [x] IR staging documented (`compiler/ir/overview.md`)
- [x] Implement MIR -> backend lowering (Cranelift-first)
- [x] Emit object files for Linux x86_64 first target
- [x] Integrate linker for executable output
- [x] Add debug info emission basics

### 2.2 Minimal Standard Library

- [x] Define Phase 2 stdlib boundaries with initial `io` contract (`print/println`)
- [x] Implement hello-world IO intrinsic path (`print`/`println` -> runtime)
- [x] Add deterministic utility APIs for contract/example execution
- [x] Document stdlib stability policy

### 2.3 CLI Tooling

- [x] Implement `vibe build`
- [x] Implement `vibe run`
- [x] Implement `vibe test`
- [x] Implement `vibe check`
- [x] Add profile/target flags (`--profile`, `--target`, `--offline`)

### Phase 2 Exit Criteria

- [x] Hello-world style programs compile and run natively (test `crates/vibe_cli/tests/phase2_native.rs` case `hello_world_build_and_run`, workflow `.github/workflows/phase2-native.yml` job `hello_world_smoke`)
- [x] Sample specs compile with basic stdlib dependencies (tests `crates/vibe_cli/tests/phase2_native.rs` cases `function_call_fixture_runs`, `if_control_flow_fixture_runs`)
- [x] CLI commands stable for local developer workflows (workflow `.github/workflows/phase2-native.yml` jobs `backend_tests`, `deterministic_build_smoke`)

---

## Phase 3: Ownership/Effect Checker + Concurrency Model

Goal: enforce safe concurrent behavior and effect correctness while preserving ergonomic syntax and automatic GC.

### 3.1 Ownership and Effect Semantics

- [x] Define ownership/aliasing rules for shared mutable state (thread-safety focused)
- [x] Implement effect inference and checking against `@effect`
- [x] Add transitive effect propagation over call graph
- [x] Add compiler diagnostics for missing/incorrect effects

### 3.2 Concurrency Runtime Model

- [x] Concurrency design drafted (`runtime/concurrency/design.md`)
- [x] Implement task scheduler (M:N work stealing baseline)
- [x] Implement typed bounded channels
- [x] Implement `select` semantics with fairness policy
- [x] Implement cancellation primitives and propagation

### 3.3 Reliability and Safety Validation

- [x] Concurrency stress tests (deadlock/contention/cancellation paths)
- [x] Race-pattern tests and static/dynamic checks where feasible
- [x] Panic/failure propagation rules validated in task scopes

### Phase 3 Exit Criteria

- [x] Concurrency primitives stable under stress workloads (workflow `.github/workflows/phase3-concurrency.yml` jobs `runtime_concurrency`, `stress_smoke`)
- [x] Effect checker catches mismatches with high signal (test `crates/vibe_cli/tests/frontend_fixtures.rs` groups `effect_ok`, `effect_err`; workflow `.github/workflows/phase3-concurrency.yml` job `compiler_safety`)
- [x] Runtime behavior aligns with safety-default guardrails (workflow `.github/workflows/phase3-concurrency.yml` jobs `concurrency_integration_smoke`, `determinism_checks`)

---

## Phase 4: Incremental Indexer + LSP Diagnostics

Goal: provide fast, local-first IDE feedback with semantic understanding and incremental updates.

### 4.1 Semantic Index Core

- [x] Indexer architecture drafted (`compiler/indexer/README.md`)
- [x] Implement symbol/reference index storage
- [x] Implement contract/effect metadata indexing
- [x] Implement file-change incremental update pipeline
- [x] Persist/reload index safely with schema versioning

### 4.2 LSP and Editor Integration

- [x] Go-to-definition/references support
- [x] Real-time diagnostics streaming
- [x] Intent/contract metadata surfacing in editor UI
- [x] Performance and stability testing in medium-size projects

### 4.3 Performance Targets

- [x] Cold index target validated on reference hardware
- [x] Single-file edit update latency target validated
- [x] Index memory overhead target validated

### Phase 4 Exit Criteria

- [x] Local IDE workflow is fast, stable, and offline-capable (workflow `.github/workflows/phase4-indexer-lsp.yml` jobs `indexer_and_lsp_unit_tests`, `phase4_cli_integration`, `medium_project_stability_smoke`)
- [x] Semantic index supports downstream AI sidecar reads (tests `crates/vibe_indexer/src/lib.rs`, `crates/vibe_lsp/src/lib.rs`; workflow `.github/workflows/phase4-indexer-lsp.yml` job `deterministic_index_snapshot`)

---

## Phase 5: AI Intent Engine + Verifier-Gated Suggestions

Goal: add AI productivity features without compromising determinism, cost, or trust.

### 5.0 Core Conformance Hardening (Carry-Over)

- [x] Implement native codegen support for `while` and `repeat` (remove phase-baseline fallback errors)
- [x] Implement true task spawning semantics for `go` in generated binaries
- [x] Implement full multi-case `select` lowering semantics (receive/after/closed/default with fairness policy)
- [x] Implement backend support for documented core forms used in official specs/examples (`List/Map` paths, member access, method-call lowering) (evidence: `crates/vibe_codegen/src/lib.rs`, `runtime/native/vibe_runtime.c`, `reports/v1/dynamic_containers_conformance.md`)
- [x] Add stable diagnostics for unsupported constructs with actionable migration/feature-status guidance
- [x] Add conformance fixtures proving runtime behavior matches `docs/spec/semantics.md` for control flow and concurrency primitives
- [x] Activate `@require/@ensure` runtime checks by default in dev/test and verify policy behavior in integration tests

### 5.1 AI Sidecar Core

- [x] Sidecar architecture drafted (`ai/sidecar/architecture.md`)
- [x] Cost model drafted (`ai/sidecar/cost_model.md`)
- [x] Offline mode drafted (`ai/sidecar/offline_mode.md`)
- [x] Implement sidecar service with local-first execution
- [x] Integrate read-only semantic index access

### 5.2 Intent Lint and Suggestions

- [x] Implement on-demand intent lint command (`vibe lint --intent`)
- [x] Implement changed-only mode (`vibe lint --intent --changed`)
- [x] Add confidence + evidence in AI diagnostics output
- [x] Ensure suggestions are verifier-gated and compiler-revalidated

### 5.3 Risk Controls

- [x] Enforce latency and cost budgets in runtime policy
- [x] Enforce non-blocking compile pipeline under AI failure/timeouts
- [x] Add policy controls for local-only / hybrid / cloud modes
- [x] Add telemetry dashboards (opt-in only)

### 5.4 Evidence Artifacts and Gating

- [x] Add Phase 5 CI workflow with explicit sidecar/lint gates (`.github/workflows/phase5-ai-sidecar.yml`)
- [x] Add compile non-blocking parity test suite (AI enabled vs disabled compile/check/build parity)
- [x] Publish Phase 5 evidence bundle (`reports/phase5/summary.md`, `reports/phase5/cost_latency.json`, `reports/phase5/intent_lint_quality.json`)
- [x] Add regression thresholds for sidecar latency/cost and intent-lint quality in CI policy

### Phase 5 Exit Criteria

- [x] Documented v0.1 semantics for core control-flow/concurrency are executable in native backend or explicitly marked as release-blocking exceptions (evidence: conformance tests + report `reports/phase5/semantics_conformance.md`)
- [x] AI features clearly improve workflow without compile dependency (evidence: workflow `.github/workflows/phase5-ai-sidecar.yml` job `non_blocking_compile`, report `reports/phase5/workflow_impact.md`)
- [x] Cost/latency budgets consistently respected (evidence: report `reports/phase5/cost_latency.json`, CI threshold gate results)
- [x] Intent lint trusted as advisory signal with low false positives (evidence: report `reports/phase5/intent_lint_quality.json` with precision/recall breakdown)

---

## Phase 6: Self-Hosting Path + Ecosystem

Goal: move from core compiler/runtime to a sustainable developer ecosystem.

### 6.0 Source Extension Migration (`.vibe` -> `.yb`)

- [x] Add dual-extension source discovery support (`.vibe` + `.yb`) in CLI/indexer/LSP/watcher pipelines (evidence: `crates/vibe_cli/src/main.rs`, `crates/vibe_indexer/src/watcher.rs`, `crates/vibe_indexer/src/layout.rs`)
- [x] Update changed-file detection and workspace scans to include both extensions (git diff globs, recursive collectors, incremental hashing) (evidence: `crates/vibe_cli/src/main.rs`, `crates/vibe_indexer/src/watcher.rs`)
- [x] Add compatibility test matrix proving command parity for both extensions (`check`, `build`, `run`, `test`, `lint`, `index`, `lsp`) (evidence: `.github/workflows/phase6-extension-parity.yml`)
- [x] Make `vibe new` and docs/spec samples default to `.yb` while preserving `.vibe` backward compatibility (evidence: `crates/vibe_cli/src/main.rs`, `docs/spec/syntax_samples.yb`, `crates/vibe_cli/tests/phase6_ecosystem.rs`)
- [x] Publish extension migration guide with deprecation timeline and opt-in warning policy (evidence: `reports/phase6/source_extension_migration.md`, `docs/migrations/v1_0_source_extension_transition.md`, `docs/policy/source_extension_policy_v1x.md`)
- [x] Define `.vibe` removal gate using adoption + CI parity thresholds (no hard removal before thresholds are met) (evidence: `docs/policy/source_extension_policy_v1x.md`)

### 6.1 Self-Hosting Roadmap

- [x] Define bootstrap strategy (host language -> VibeLang compiler transition) (evidence: `reports/phase6/bootstrap_strategy.md`)
- [x] Build milestone plan for partial then full self-hosting (evidence: `reports/phase6/self_hosting_milestones.md`)
- [x] Establish conformance tests to compare bootstrap vs self-host behavior (evidence: `crates/vibe_fmt/tests/selfhost_conformance.rs`, `reports/phase6/self_hosting_conformance.md`)

### 6.2 Package and Build Ecosystem

- [x] Package manager design and lockfile format (evidence: `docs/package/vibe_toml_spec.md`, `docs/package/vibe_lock_spec.md`)
- [x] Dependency resolution and reproducible install (evidence: `crates/vibe_pkg/src/lib.rs`, `cargo test -p vibe_pkg`)
- [x] Registry/mirror strategy (including offline workflows) (evidence: `vibe pkg install --mirror`, `reports/phase6/package_manager_foundation.md`)

### 6.3 Developer Experience Tooling

- [x] Formatter (`vibe fmt`) (evidence: `crates/vibe_fmt/src/lib.rs`, `crates/vibe_cli/src/main.rs`)
- [x] Docs generator (`vibe doc`) (evidence: `crates/vibe_doc/src/lib.rs`, `crates/vibe_cli/src/main.rs`)
- [x] Unified test runner with contract/example integration (evidence: `crates/vibe_cli/src/main.rs` `run_test`, `crates/vibe_cli/tests/phase2_native.rs`)
- [x] Project scaffolding command (`vibe new`) (evidence: `crates/vibe_cli/src/main.rs`, `crates/vibe_cli/tests/phase6_ecosystem.rs`)

### 6.4 Adoption and Stability

- [x] Versioning and compatibility policy (evidence: `docs/policy/versioning_compatibility.md`)
- [x] Release pipeline and changelog process (evidence: `.github/workflows/release.yml`, `docs/release/process.md`, `CHANGELOG.md`)
- [x] Migration guides between language/toolchain versions (evidence: `docs/migrations/TEMPLATE.md`, `docs/migrations/v1_0_source_extension_transition.md`)
- [x] Define source-extension compatibility policy (`.vibe` legacy, `.yb` canonical) for v1.x (evidence: `docs/policy/source_extension_policy_v1x.md`)

### 6.5 Portability and Conformance Governance

- [x] Expand native target support toward charter targets (Linux arm64, macOS arm64) with parity checklist (evidence: `crates/vibe_runtime/src/lib.rs`, `crates/vibe_codegen/src/lib.rs`)
- [x] Add cross-target CI matrix for build/run determinism and runtime smoke tests (evidence: `.github/workflows/phase6-portability.yml`)
- [x] Publish target-tier support matrix (feature coverage, performance expectations, known limitations) (evidence: `docs/targets/support_matrix.md`)
- [x] Require phase-complete evidence: spec-to-runtime conformance tests + CI gates before marking a phase done (evidence: `.github/workflows/phase6-portability.yml`, `.github/workflows/release.yml`, `reports/phase6/phase6_exit_evidence.md`)
- [x] Maintain a release-visible limitations/debt register with owner, severity, and target phase (evidence: `docs/targets/limitations_register.md`)

### Phase 6 Exit Criteria

- [x] Teams can build, test, publish, and maintain VibeLang projects end-to-end (evidence: `.github/workflows/release.yml`, `reports/phase6/package_manager_foundation.md`, `crates/vibe_cli/tests/phase6_ecosystem.rs`)
- [x] Self-hosting path is demonstrated or scheduled with validated milestones (evidence: bootstrap-vs-self-host conformance report in `reports/phase6/self_hosting_conformance.md`)
- [x] Target-tier support and conformance governance process are operational in release workflow (evidence: `.github/workflows/phase6-portability.yml`, `.github/workflows/release.yml`, `docs/targets/support_matrix.md`)
- [x] `.yb` is canonical for new projects while `.vibe` remains regression-free during migration (evidence: dual-extension parity CI + report `reports/phase6/source_extension_migration.md`)

---

## Cross-Phase Tracking Metrics

- [x] Compile performance dashboard (clean/no-op/incremental) (evidence: `reports/phase6/metrics/phase6_metrics.json`)
- [x] Runtime performance dashboard (GC pauses, throughput, latency) (evidence: `reports/phase6/metrics/phase6_metrics.json`)
- [x] Contract coverage and failure signal quality (evidence: `reports/phase6/metrics/phase6_metrics.json`)
- [x] Intent lint precision/recall (where measurable) (evidence: `reports/phase6/metrics/phase6_metrics.json`)
- [x] Developer productivity indicators (time-to-first-binary, median feedback loop) (evidence: `reports/phase6/metrics/phase6_metrics.json`)
- [x] Spec conformance dashboard (documented constructs vs runtime-validated coverage) (evidence: `reports/phase6/metrics/phase6_metrics.json`)
- [x] Unsupported-feature backlog trend (count, severity, time-to-resolution) (evidence: `reports/phase6/metrics/phase6_metrics.json`)
- [x] Cross-target compatibility pass rate by profile/target (evidence: `reports/phase6/metrics/phase6_metrics.json`, `.github/workflows/phase6-portability.yml`)
- [x] Source-extension adoption ratio (`.yb` vs `.vibe`) and dual-support parity pass rate (evidence: `reports/phase6/metrics/phase6_metrics.json`)

## Current Status Snapshot

- Core strategy/spec docs: drafted and phase-1 through phase-6 policy docs are now published (package/release/targets/migration/self-host reports)
- Implementation code: Phase 6 ecosystem baseline is delivered (`vibe pkg`, `vibe fmt`, `vibe doc`, `vibe new`, selfhost seed, cross-target governance updates)
- Verification: phase1-6 workflows and local validation suites are green (including clippy, parity tests, selfhost conformance, and metrics thresholds)
- Next execution focus: execute the ordered post-phase-6 readiness plan (test corpus + sample programs, GitHub README, v1 release tightening, and book-quality documentation roadmap)

---

## Phase 7: V1 Readiness Execution Plan (Ordered)

Goal: convert the current strong engineering baseline into a polished, public, production-ready v1 launch path.

Execution order is fixed and should be followed top to bottom.

### 7.1 Ordered Item 1 — Comprehensive Language Validation + Sample Programs

#### 7.1.a Test Corpus Structure (Basic -> Advanced)

- [x] Define fixture taxonomy and naming convention for progression levels (`basic`, `intermediate`, `advanced`, `stress`) under `compiler/tests/fixtures/phase7/` (evidence: `compiler/tests/fixtures/phase7/README.md`, level READMEs under `phase7/basic|intermediate|advanced|stress`)
- [x] Add minimal syntax/lexing fixtures: literals, comments, whitespace, indentation boundaries, unary/binary operators, grouping (evidence: fixtures `phase7/basic/syntax/*.yb`, test `crates/vibe_cli/tests/frontend_fixtures.rs` `phase7_basic_and_intermediate_matrix`)
- [x] Add identifier fixtures: valid identifiers, reserved keyword rejection, shadowing behavior, naming edge cases (evidence: fixtures `phase7/basic/identifiers/*.yb` + `.diag`, test `crates/vibe_cli/tests/frontend_fixtures.rs`)
- [x] Add parser-recovery fixtures: malformed blocks/annotations with multi-error stability expectations (evidence: fixtures `phase7/basic/parser_recovery/*.yb` + `.diag`, deterministic test `phase7_frontend_outputs_are_deterministic`)
- [x] Add type-check fixtures: inference boundaries, mismatch diagnostics, unknown symbol/function handling, deterministic error ordering (evidence: fixtures `phase7/basic/typecheck/*.yb` + `.diag`, tests `phase7_basic_and_intermediate_matrix`, `phase7_frontend_outputs_are_deterministic`)

#### 7.1.b Annotation/Contract/Intent Coverage

- [x] Add dedicated fixtures for `@intent`, `@examples`, `@require`, `@ensure`, `@effect` individually and in valid combinations (evidence: fixtures `phase7/intermediate/annotations/annotations__all_valid_combo.yb`, `phase7/advanced/intent_validation/*.yb`)
- [x] Add invalid-annotation fixtures (unknown tags, malformed payloads, wrong placement) with stable diagnostics (evidence: fixtures `phase7/intermediate/annotations/annotations__unknown_tag.yb`, `annotations__intent_missing_string.yb`, `annotations__wrong_position.yb` + `.diag`)
- [x] Add runtime contract policy tests covering dev/test default behavior and explicit overrides (evidence: tests `crates/vibe_cli/tests/phase2_native.rs` `vibe_test_enforces_contract_runtime_checks_by_default`, `vibe_test_can_disable_contract_runtime_checks_with_env_override`)
- [x] Add `@examples` correctness tests for pass/fail reporting quality (function, input, expected/actual, source span) (evidence: fixtures with examples in `phase7/intermediate/annotations` and `phase7/advanced/intent_validation`, test `crates/vibe_cli/tests/phase7_validation.rs` `phase7_language_tour_contract_examples_pass_in_vibe_test`)
- [x] Add effect conformance tests proving `@effect` declarations align with observed behavior (evidence: fixtures `annotations__effect_declared_match.yb`, `annotations__effect_drift.yb` + `.diag`, lint drift fixture `intent_validation__effect_drift.yb`)

#### 7.1.c Single-Threaded Program Suite

- [x] Add canonical single-thread sample programs: hello world, calculator, collection transform pipeline, small state machine (evidence: fixtures `phase7/advanced/single_thread/single_thread__hello_world.yb`, `single_thread__calculator.yb`, `single_thread__pipeline_transform.yb`, `single_thread__state_machine.yb`)
- [x] Add deterministic output assertions for repeated runs (same input => identical stdout and exit code) (evidence: test `crates/vibe_cli/tests/phase7_validation.rs` `phase7_single_thread_samples_run_expected_outputs`)
- [x] Add build artifacts determinism checks for each sample (`.o`, binary hash, debug map stability) (evidence: test `crates/vibe_cli/tests/phase7_validation.rs` `phase7_single_thread_build_artifacts_are_deterministic`)
- [x] Add small "language tour" sample showing functions, control flow, contracts, and examples in one file (evidence: fixture `phase7/advanced/single_thread/single_thread__language_tour.yb`, test `phase7_language_tour_contract_examples_pass_in_vibe_test`)

#### 7.1.d Multi-Threaded/Concurrency Program Suite

- [x] Add bounded worker-pool sample using `go`, `chan`, `select`, cancellation token propagation (evidence: fixture `phase7/advanced/concurrency/concurrency__worker_pool.yb`, test `crates/vibe_cli/tests/phase7_concurrency.rs`)
- [x] Add fan-in/fan-out sample with fairness assertions on `select` (evidence: fixtures `phase7/advanced/concurrency/concurrency__fan_in.yb`, `concurrency__fan_out.yb`, deterministic checks in `phase7_concurrency_samples_are_deterministic_and_bounded`)
- [x] Add timeout/retry sample using `after` branch behavior in `select` (evidence: fixture `phase7/advanced/concurrency/concurrency__timeout_retry.yb`, expected output assertion in `phase7_concurrency_samples_run_expected_outputs`)
- [x] Add concurrency stress scenario fixtures with deterministic pass/fail criteria and bounded runtime (evidence: fixture `phase7/stress/concurrency/concurrency__bounded_stress.yb`, bounded-time assertion in `phase7_concurrency_samples_are_deterministic_and_bounded`)
- [x] Add negative tests for concurrency misuse with actionable diagnostics (evidence: fixtures `phase7/advanced/concurrency_err/*.yb` + `.diag`, test `phase7_concurrency_negative_fixtures_match_golden`)

#### 7.1.e Intent-Driven Development Validation

- [x] Add intent lint fixtures for "good intent matches implementation" and "intent drift" cases (evidence: fixtures `phase7/advanced/intent_validation/*.yb`, test `crates/vibe_cli/tests/phase7_intent_validation.rs` `intent_lint_detects_good_match_vs_drift_cases`)
- [x] Add changed-only lint mode validation for both git-present and no-git flows (evidence: test `intent_lint_changed_mode_supports_no_git_and_git_present_flows`)
- [x] Add verifier-gated suggestion tests ensuring rejected suggestions never alter compile determinism (evidence: test `intent_lint_verifier_gate_rejects_invalid_suggestions`)
- [x] Add intent lint quality scoring harness update (`precision`, `recall`, `false-positive trend`) with report output (evidence: `tooling/metrics/collect_intent_lint_quality.py`, `tooling/metrics/validate_intent_lint_quality.py`, report `reports/phase7/intent_lint_quality_trend.json`)

#### 7.1.f CI and Evidence for Item 1

- [x] Add dedicated workflow `.github/workflows/phase7-language-validation.yml` for corpus execution (evidence: workflow jobs `frontend_corpus_matrix`, `single_thread_determinism`, `concurrency_advanced_and_stress`, `intent_validation_and_quality_trend`)
- [x] Publish report `reports/phase7/language_validation_matrix.md` with pass/fail matrix (feature x test level) (evidence: `reports/phase7/language_validation_matrix.md`)
- [x] Publish report `reports/phase7/sample_programs_catalog.md` with run/build/test commands and expected outputs (evidence: `reports/phase7/sample_programs_catalog.md`)

### 7.2 Ordered Item 2 — GitHub README (Product-Grade)

#### 7.2.a README Content and Positioning

- [x] Rewrite root `README.md` with product-quality structure and clear sections (evidence: `README.md` with TOC, architecture, install, quickstart, troubleshooting, roadmap, contribution sections)
- [x] Add concise project pitch: what VibeLang is and why it was built (evidence: `README.md` section `Project Pitch`)
- [x] Add "What VibeLang solves today" section with concrete, current capabilities (evidence: `README.md` section `What VibeLang Solves Today`)
- [x] Add "What is experimental / in-progress" section to set user expectations honestly (evidence: `README.md` section `What Is Experimental / In Progress`)
- [x] Add use-case section (systems tooling, concurrent services, deterministic build pipelines, intent-aware development) (evidence: `README.md` section `Use Cases`)

#### 7.2.b Quickstart and Installation UX

- [x] Add installation paths: from source, local binary usage, and future packaged release placeholder (evidence: `README.md` section `Installation`)
- [x] Add 60-second quickstart (`vibe new`, `vibe run`, `vibe test`, `vibe fmt`, `vibe doc`) (evidence: `README.md` section `60-Second Quickstart`)
- [x] Add hello-world + one contract/intent sample snippet (evidence: `README.md` section `Code Samples`)
- [x] Add troubleshooting section (common setup/compiler toolchain issues) (evidence: `README.md` section `Troubleshooting`)

#### 7.2.c Visual and Navigation Quality

- [x] Add section anchors/table of contents for easy scanning (evidence: `README.md` section `Table of Contents`)
- [x] Add architecture diagram (compiler core, runtime, indexer/lsp, sidecar boundaries) (evidence: `README.md` section `Architecture`)
- [x] Add roadmap snapshot links to `docs/development_checklist.md` and phase reports (evidence: `README.md` section `Roadmap Snapshot`)
- [x] Add contribution/start-here section (build, test, lint, CI expectations) (evidence: `README.md` section `Contributing: Start Here`)

#### 7.2.d README Validation

- [x] Add markdown lint/link check for README in CI (evidence: workflow `.github/workflows/phase7-readme-quality.yml`, job `readme_markdown_and_links`)
- [x] Add quickstart smoke check in CI to ensure commands in README stay executable (evidence: workflow `.github/workflows/phase7-readme-quality.yml`, job `readme_quickstart_smoke`)

### 7.3 Ordered Item 3 — Tightened V1 Production Release Checklist

#### 7.3.a Scope Freeze and Release Gates

- [x] Define explicit v1 feature scope freeze list and non-goals list (evidence: `docs/release/v1_scope_freeze.md`)
- [x] Convert all remaining top-level unchecked guardrails into owned release gates (owner/severity/target milestone) (evidence: `docs/release/v1_release_gates.md`)
- [x] Add release blocker policy (`P0`/`P1` criteria) and merge gate alignment (evidence: `docs/release/release_blocker_policy.md`)
- [x] Create v1 release readiness dashboard report (`reports/v1/readiness_dashboard.md`) (evidence: `reports/v1/readiness_dashboard.md`)

#### 7.3.b Engineering Quality Hardening

- [ ] Close remaining determinism/safety/performance unchecked items in this checklist with evidence
- [x] Define minimum test coverage expectations for parser/type/runtime/cli/intent-lint paths (evidence: `docs/testing/coverage_policy.md`, `tooling/metrics/validate_phase7_coverage_matrix.py`)
- [x] Add long-run stability/soak tests with bounded budgets and pass thresholds (evidence: `crates/vibe_cli/tests/phase7_v1_tightening.rs`, `reports/v1/quality_budgets.json`, workflow `.github/workflows/v1-release-gates.yml` job `quality_and_coverage_gate`)
- [x] Add packaging integrity checks (checksums/signatures/provenance plan) (evidence: workflow `.github/workflows/v1-release-gates.yml` job `packaging_integrity_smoke`)
- [x] Add upgrade/downgrade compatibility test path between adjacent v1.x versions (evidence: workflow `.github/workflows/v1-release-gates.yml` job `compatibility_gate`)

#### 7.3.c Operational Readiness

- [x] Define release candidate process (`rc1`, `rc2`, promote/reject criteria) (evidence: `docs/release/rc_process.md`)
- [x] Define rollback playbook for bad release detection and mitigation (evidence: `docs/release/rollback_playbook.md`)
- [x] Define issue triage SLA and bug severity taxonomy for public users (evidence: `docs/support/issue_triage_sla.md`)
- [x] Define telemetry/privacy statement for optional AI-related signals (evidence: `docs/privacy/telemetry_statement.md`)
- [x] Add "known limitations" publication gate before each release (evidence: `docs/release/known_limitations_gate.md`)

#### 7.3.d CI/Reporting for V1 Tightening

- [x] Add workflow `.github/workflows/v1-release-gates.yml` for consolidated v1 blocking checks (evidence: `.github/workflows/v1-release-gates.yml`)
- [x] Publish `reports/v1/release_candidate_checklist.md` template and first run (evidence: `reports/v1/release_candidate_checklist.md`, `reports/v1/smoke_validation.md`)
- [x] Require all v1 gate reports linked in release PR description (evidence: `.github/pull_request_template.md`, workflow `.github/workflows/v1-release-gates.yml` job `release_pr_report_links_gate`)

#### 7.3.e Compiler Self-Host Readiness (First)

- [x] Promote self-host milestone M1 from scheduled to release-gated execution (selfhost formatter executable path in CI with host fallback retained) (evidence: `.github/workflows/v1-release-gates.yml` job `selfhost_readiness_gate`, `selfhost/formatter_core.yb`, `docs/selfhost/m1_formatter_contract.md`)
- [x] Add bootstrap-vs-selfhost deterministic parity gate for release candidates (byte-for-byte output + repeat-run stability) (evidence: `crates/vibe_fmt/tests/selfhost_conformance.rs`, `reports/v1/selfhost_readiness.md`)
- [x] Define and implement one self-host compiler/frontend slice in shadow mode (`M3` readiness starter) to prove compiler internals can be authored in VibeLang (evidence: `selfhost/diagnostics_ordering_shadow.yb`, `crates/vibe_diagnostics/tests/selfhost_shadow_ordering.rs`, `reports/v1/selfhost_readiness.md`)
- [x] Publish `reports/v1/selfhost_readiness.md` with milestone status, parity metrics, fallback toggles, and go/no-go criteria (evidence: `reports/v1/selfhost_readiness.md`, `reports/v1/selfhost_readiness.json`)
- [x] Add blocking self-host readiness job in `.github/workflows/v1-release-gates.yml` (evidence: workflow job `selfhost_readiness_gate` and `summary` dependency wiring)
- [x] Exit gate: `7.3.f` language-surface expansion does not close until `7.3.e` has one successful RC dry-run evidence cycle (evidence: `reports/v1/selfhost_readiness.md` run counter + `rc1-dryrun-local`; `7.3.f` remains unchecked)

#### 7.3.f Language Surface + Dynamic Runtime Data Structures (Second, After 7.3.e)

##### 7.3.f.0 Spec Completeness Sub-Gate (Required Before 7.3.f Runtime Closeout)

- [x] Publish normative spec architecture and source-of-truth taxonomy for production docs (`docs/spec/README.md`, `docs/spec/spec_glossary.md`, `docs/spec/spec_decision_log.md`, `docs/spec/grammar_v1_0.ebnf`)
- [x] Reconcile syntax/semantics contradictions (`match`/`break`/`continue`, optional typing, contract placement) and add compatibility appendix (evidence: `docs/spec/syntax.md`, `docs/spec/semantics.md`, `docs/spec/phase1_resolved_decisions.md`)
- [x] Publish normative language reference docs for type system, numerics, mutability, strings/containers, control flow, concurrency/async/thread model, memory/error/ABI/module boundaries (evidence: docs under `docs/spec/*.md` created for each surface)
- [x] Add spec traceability matrix mapping normative rules to tests/deferred items (evidence: `docs/spec/spec_coverage_matrix.md`)
- [x] Add spec consistency and coverage validators and wire blocking `spec_integrity_gate` in v1 workflow (evidence: `tooling/spec/validate_spec_consistency.py`, `tooling/spec/validate_spec_coverage.py`, workflow `.github/workflows/v1-release-gates.yml` job `spec_integrity_gate`)
- [x] Add spec readiness evidence artifact and dashboard linkage (evidence: `reports/v1/spec_readiness.md`, `reports/v1/readiness_dashboard.md`)

##### 7.3.f.1 Runtime + Compiler Implementation Closeout (Still Blocking GA)

- [x] Define v1 language-surface completion scope (dynamic `Str`/`List`/`Map`, required keywords, literal forms, and container API behavior) with determinism/ordering guarantees (evidence: `docs/spec/containers.md` section `7.3.f.1 Implementation Support Freeze (v1 GA Blocker Scope)`, `docs/spec/strings_and_text.md`, `docs/spec/type_system.md`, `docs/spec/numeric_model.md`)
- [x] Add parser/type-check coverage for remaining keyword/literal/container forms with deterministic diagnostics and migration guidance (evidence: `compiler/tests/fixtures/type_ok/container_methods_basic.yb`, `compiler/tests/fixtures/type_err/map_key_mismatch.yb`, `compiler/tests/fixtures/parse_err/map_missing_colon.yb`, `crates/vibe_cli/tests/frontend_fixtures.rs`)
- [x] Add MIR/container IR operations for dynamic construction, append/concat, indexing, and iteration (evidence: `crates/vibe_mir/src/lib.rs`, `compiler/tests/fixtures/snapshots/container_ops_sample.yb`, `crates/vibe_cli/tests/frontend_fixtures.rs` test `snapshots_container_ops_mir_is_deterministic`)
- [x] Implement runtime ABI intrinsics and allocation strategy for container operations with deterministic behavior (evidence: `runtime/native/vibe_runtime.c`, `reports/v1/dynamic_containers_conformance.md`)
- [x] Implement native codegen lowering for container/member forms currently covered by `E3401`/`E3402` fallbacks (evidence: `crates/vibe_codegen/src/lib.rs`, `crates/vibe_cli/tests/phase7_v1_tightening.rs`)
- [x] Add concurrency/sendability safety checks for container values crossing `go` boundaries (evidence: `compiler/tests/fixtures/phase7/stress/ownership/ownership__list_map_sendable.yb`, `compiler/tests/fixtures/ownership_err/map_non_sendable_value_in_go.yb`)
- [x] Add algorithmic conformance fixtures requiring dynamic containers (e.g., generate parentheses backtracking) with deterministic output tests (evidence: `compiler/tests/fixtures/phase7/stress/algorithmic/algorithmic__generate_parentheses_count.yb`, `crates/vibe_cli/tests/phase7_v1_tightening.rs`)
- [x] Add container-heavy memory/GC observability checks and bounded leak-test lane evidence (evidence: `compiler/tests/fixtures/phase7/stress/memory/memory__container_pressure_loop.yb`, `crates/vibe_cli/tests/phase7_v1_tightening.rs`)
- [x] Publish `reports/v1/dynamic_containers_conformance.md` and wire a blocking CI gate in `.github/workflows/v1-release-gates.yml` (evidence: `reports/v1/dynamic_containers_conformance.md`, workflow job `dynamic_containers_gate`)

### 7.4 Ordered Item 4 — VibeLang Book + Full Documentation Program

#### 7.4.a Documentation Architecture

- [ ] Define docs information architecture: tutorials, language book, reference, tooling guides, internals
- [ ] Choose docs engine and repo layout (e.g., mdBook-style structure under `book/`)
- [ ] Define versioning strategy for docs (`latest`, `v1.x`, archived versions)
- [ ] Define docs style guide (tone, examples, conventions, glossary)

#### 7.4.b "The VibeLang Book" Chapter Checklist (Rust-Book Quality Target)

- [ ] Chapter 1: Getting started and mental model
- [ ] Chapter 2: Core syntax and semantics
- [ ] Chapter 3: Types, functions, and errors
- [ ] Chapter 4: Contracts (`@require/@ensure`) and executable examples (`@examples`)
- [ ] Chapter 5: Effects and performance reasoning
- [ ] Chapter 6: Concurrency (`go`, `chan`, `select`, cancellation)
- [ ] Chapter 7: Tooling (`check/build/run/test/fmt/doc/pkg/lint/index/lsp`)
- [ ] Chapter 8: Intent-driven development and sidecar model
- [ ] Chapter 9: Migration and compatibility policies (`.vibe` -> `.yb`, v1.x evolution)
- [ ] Chapter 10: Advanced internals overview (compiler pipeline, runtime model, indexer/LSP architecture)

#### 7.4.c Docs Quality and Automation

- [ ] Add executable code-snippet tests for all tutorial/reference pages
- [ ] Add link-check, spell-check, and stale-example CI gates
- [ ] Add "docs coverage" metric (language features documented vs implemented)
- [ ] Add periodic docs usability review checklist (new user walkthroughs)
- [ ] Publish `reports/docs/documentation_quality.md` per release

### Phase 7 Exit Criteria (Ordered Plan Complete)

- [x] Item 1 completed: comprehensive language validation matrix and sample program catalog are green with reproducible evidence (`workflow .github/workflows/phase7-language-validation.yml`, reports under `reports/phase7/`)
- [x] Item 2 completed: README is public-ready, accurate, and CI-validated against command drift (`README.md`, workflow `.github/workflows/phase7-readme-quality.yml`)
- [ ] Item 3 completed: v1 production release gates are explicitly defined, owned, and passing for at least one release-candidate cycle (evidence path: `workflow .github/workflows/v1-release-gates.yml`, `reports/v1/readiness_dashboard.md`, `reports/v1/release_candidate_checklist.md`, `reports/v1/smoke_validation.md`, `reports/v1/spec_readiness.md`; remaining blockers tracked in `reports/v1/readiness_dashboard.md`)
- [ ] Item 4 completed: book/docs program is structured, CI-gated, and includes tested chapter examples across core language/tooling surfaces (`book/`, docs CI jobs, `reports/docs/documentation_quality.md`)

---

## Phase 8: Independent Installation + CLI Maturity (No-Cargo End-User Path)

Goal: make VibeLang installable/runnable like mainstream languages on end-user machines without requiring Rust/Cargo at install time.

### 8.1 Distribution Scope and Policy

- [x] Define packaged distribution matrix and tier policy (Linux tarball/deb/rpm, macOS pkg/Homebrew, Windows zip/msi) (evidence: `docs/release/distribution_matrix.md`)
- [x] Define artifact trust policy (checksums, signatures, provenance/SBOM requirements) (evidence: `docs/release/distribution_security.md`)
- [x] Define offline/air-gapped install + mirror strategy for packaged binaries (evidence: `docs/release/offline_install_policy.md`)
- [x] Define update channels and compatibility policy for installer paths (`stable`, `rc`, rollback channel) (evidence: `docs/policy/install_channels_v1.md`)

### 8.2 Packaging and Release Automation

- [x] Add workflow `.github/workflows/v1-packaged-release.yml` to produce standalone `vibe` artifacts for tier-1 targets (evidence: workflow `.github/workflows/v1-packaged-release.yml` job `package_artifacts`)
- [x] Add artifact signing/checksum/SBOM generation and publication in CI release jobs (evidence: workflow `.github/workflows/v1-packaged-release.yml` job `sign_attest_and_sbom`)
- [x] Add reproducibility checks between release candidates for packaged binaries (hash/metadata stability policy) (evidence: workflow `.github/workflows/v1-packaged-release.yml` job `packaged_reproducibility`, tooling `tooling/release/checksum_manifest.py`, baseline `reports/v1/reproducibility/last_rc_checksums.json`, report `reports/v1/phase8_ci_evidence.md`)
- [x] Add install-smoke jobs on clean runners that do not assume Rust/Cargo on PATH (evidence: workflow `.github/workflows/v1-packaged-release.yml` jobs `install_smoke_linux`, `install_smoke_macos`, `install_smoke_windows`)

### 8.3 End-User Install UX (No Cargo Required)

- [x] Publish install docs for each supported platform with copy-paste commands (evidence: `docs/install/linux.md`, `docs/install/macos.md`, `docs/install/windows.md`)
- [x] Add install verification flow (`vibe --version`, hello-world run, uninstall path) (evidence: `docs/install/verification.md`)
- [x] Add update + rollback instructions for packaged installs (evidence: `docs/install/update_and_rollback.md`)
- [x] Add troubleshooting matrix for installer/network/signature failures (evidence: `docs/install/troubleshooting.md`)

### 8.4 CLI Maturity and Discoverability

- [x] Promote root `vibe --help` into a full manual-style command reference with examples and error-guided next steps (evidence: `docs/cli/help_manual.md`, `crates/vibe_cli/src/main.rs`)
- [x] Add per-command help quality bar (`vibe <command> --help` has usage, flags, examples, exit behavior) and drift tests (evidence: test `crates/vibe_cli/tests/cli_help_snapshots.rs`)
- [x] Add stable `vibe --version` output policy (semver, commit, target, profile) and machine-readable mode (evidence: `docs/cli/version_output.md`, test `crates/vibe_cli/tests/cli_version.rs`)
- [x] Add CI gate for help/version UX regression (evidence: workflow `.github/workflows/v1-cli-ux.yml`)

### 8.5 Reporting and Gate Wiring

- [x] Publish `reports/v1/install_independence.md` with clean-machine install/run evidence (evidence: `reports/v1/install_independence.md`)
- [x] Publish `reports/v1/distribution_readiness.md` with platform matrix status and known limitations (evidence: `reports/v1/distribution_readiness.md`)
- [x] Wire blocking `independent_install_gate` into `.github/workflows/v1-release-gates.yml` summary dependency (evidence: workflow `.github/workflows/v1-release-gates.yml` job `independent_install_gate`, `summary.needs`)

### Phase 8 Exit Criteria

- [x] A fresh machine without Rust/Cargo can install `vibe` from packaged artifacts and run programs successfully (evidence: `reports/v1/install_independence.md`, `reports/v1/phase8_ci_evidence.md`, workflow `.github/workflows/v1-packaged-release.yml` jobs `install_smoke_linux`, `install_smoke_macos`, `install_smoke_windows`)
- [x] `vibe --help` and `vibe --version` are stable, documented, and CI-regression-tested (evidence: `docs/cli/help_manual.md`, `docs/cli/version_output.md`, `reports/v1/phase8_ci_evidence.md`, workflow `.github/workflows/v1-cli-ux.yml` jobs `cli_help_and_version_regressions`, `cli_docs_presence`)
- [x] Packaged release artifacts are signed, checksummed, and policy-compliant for tier-1 targets (evidence: `reports/v1/distribution_readiness.md`, `reports/v1/phase8_ci_evidence.md`, workflow `.github/workflows/v1-packaged-release.yml` jobs `packaged_reproducibility`, `sign_attest_and_sbom`)

### 8.6 Linux Installer Compatibility Follow-Up (Post-Close)

- [x] Ensure Linux packaged binary compatibility for common Ubuntu/WSL baselines (glibc 2.35+) or publish static `musl` artifact fallback (evidence: workflow `.github/workflows/v1-packaged-release.yml` Linux GNU baseline build on `ubuntu-22.04`, step `verify linux glibc compatibility baseline`, policy `docs/release/linux_runtime_compatibility_policy.md`)
- [x] Add blocking CI lane that executes packaged Linux artifact on both `ubuntu-22.04` and latest Ubuntu runner to catch loader/runtime ABI drift before release (evidence: workflow `.github/workflows/v1-packaged-release.yml` jobs `install_smoke_linux`, `install_smoke_linux_latest`; workflow `.github/workflows/v1-release-gates.yml` job `linux_compatibility_gate`)
- [x] Document minimum Linux runtime requirements and fallback install path in `docs/install/linux.md` and release notes (evidence: `docs/install/linux.md`, `docs/install/troubleshooting.md`, `docs/release/linux_runtime_compatibility_policy.md`)

---

## Phase 9: Progressive Self-Host Transition (M2 -> M3 Expansion -> M4 Default Switch)

Goal: safely move from host-implemented compiler/tooling components to VibeLang-authored components with deterministic parity, rollback controls, and production confidence.

### 9.1 M2 Self-Host Expansion (Tooling Components)

- [x] Port docs/diagnostics formatter components to VibeLang with byte-for-byte parity harnesses (evidence: `selfhost/docs_formatter_core.yb`, `selfhost/diagnostics_formatter_core.yb`, fixture parity tests in `crates/vibe_doc/tests/selfhost_conformance.rs`, `crates/vibe_diagnostics/tests/selfhost_formatter_conformance.rs`)
- [x] Add deterministic repeat-run tests for M2 components across fixture corpus (evidence: tests `selfhost_docs_formatter_repeat_runs_are_deterministic`, `selfhost_diagnostics_formatter_repeat_runs_are_deterministic`)
- [x] Publish M2 contract and component boundaries (evidence: `docs/selfhost/m2_formatter_diagnostics_contract.md`)
- [x] Publish M2 readiness evidence (evidence: `reports/v1/selfhost_m2_readiness.md`, workflow `.github/workflows/v1-release-gates.yml` job `selfhost_m2_gate`)

### 9.2 M3 Expansion (Compiler Frontend Slices in Shadow Mode)

- [x] Expand M3 from starter shadow slice to multiple frontend slices (parser diagnostics normalization, type diagnostic ordering, selected MIR formatting paths) (evidence: `selfhost/frontend_shadow_slices.yb`, fixtures `selfhost/fixtures/m3_parser_diag_normalization.*`, `selfhost/fixtures/m3_type_diag_ordering.*`, `selfhost/fixtures/m3_mir_formatting.*`)
- [x] Run host + self-host shadow dual-path checks in CI and block on parity drift (evidence: test `crates/vibe_cli/tests/selfhost_m3_expansion.rs`, workflow `.github/workflows/v1-release-gates.yml` job `selfhost_m3_shadow_gate`, artifact `v1-selfhost-m3-shadow`)
- [x] Track and enforce shadow-mode performance budgets (latency/memory overhead ceilings) (evidence: test `m3_shadow_performance_budgets_are_within_thresholds` in `crates/vibe_cli/tests/selfhost_m3_expansion.rs`, workflow `.github/workflows/v1-release-gates.yml` job `selfhost_m3_shadow_gate`)
- [x] Publish expanded M3 readiness evidence (evidence: `reports/v1/selfhost_m3_expansion.md`)

### 9.3 M4 Transition Gate (Default Switch Strategy)

- [x] Define graduation criteria for switching selected components to self-host default path (evidence: `docs/selfhost/m4_transition_criteria.md`, first-wave candidate `diagnostics ordering`)
- [x] Implement explicit rollback/fallback toggles for each promoted component (host path remains immediately available) (evidence: `VIBE_DIAGNOSTICS_SORT_MODE`, `VIBE_SELFHOST_FORCE_HOST_FALLBACK`, `crates/vibe_diagnostics/tests/selfhost_transition_toggle.rs`)
- [x] Run at least one release-candidate cycle with promoted self-host default component(s) and no parity regressions (evidence: workflow `.github/workflows/v1-release-gates.yml` job `selfhost_m4_rc_cycle_gate`, artifact `v1-selfhost-m4-rc-cycle`, `reports/v1/release_candidate_checklist.md`)
- [x] Publish self-host transition playbook (evidence: `docs/release/selfhost_transition_playbook.md`)

### 9.4 Governance, Ownership, and Reporting

- [x] Extend `reports/v1/selfhost_readiness.md` to track M1/M2/M3/M4 component-level parity counters and ownership (evidence: `reports/v1/selfhost_readiness.md`, `reports/v1/selfhost_readiness.json`)
- [x] Publish per-component ownership + signoff matrix (evidence: `docs/selfhost/component_ownership.md`)
- [x] Add blocking `selfhost_transition_gate` in `.github/workflows/v1-release-gates.yml` covering M2/M3/M4 evidence (evidence: workflow job `selfhost_transition_gate`, artifact `v1-selfhost-transition-gate`)
- [x] Add PR template requirements for self-host parity artifacts in release candidate branches (evidence: `.github/pull_request_template.md`, workflow `.github/workflows/v1-release-gates.yml` job `release_pr_report_links_gate`)

### Phase 9 Exit Criteria

- [x] VibeLang team can author and ship meaningful compiler/tooling logic in VibeLang on a regular path (not seed-only) with CI parity gates (evidence: workflows `.github/workflows/v1-release-gates.yml` jobs `selfhost_m2_gate`, `selfhost_m3_shadow_gate`, `selfhost_transition_gate`; reports `reports/v1/selfhost_m2_readiness.md`, `reports/v1/selfhost_m3_expansion.md`)
- [x] M2 and expanded M3 components show deterministic parity in consecutive CI runs above agreed threshold (evidence: `reports/v1/selfhost_readiness.md`, `reports/v1/selfhost_m2_readiness.md`, `reports/v1/selfhost_m3_expansion.md`)
- [x] At least one component is promoted to self-host default path with proven rollback and successful RC cycle evidence (evidence: `docs/release/selfhost_transition_playbook.md`, `reports/v1/release_candidate_checklist.md`, workflow `.github/workflows/v1-release-gates.yml` job `selfhost_m4_rc_cycle_gate`)
- [x] Self-host transition is production-safe: fallback controls, ownership, and incident-response procedures are documented and exercised (evidence: `docs/selfhost/component_ownership.md`, `docs/selfhost/m4_transition_criteria.md`, `docs/release/selfhost_transition_playbook.md`, test `crates/vibe_diagnostics/tests/selfhost_transition_toggle.rs`)

---

## Production Readiness Delta Snapshot (Audit: 2026-02-21)

What is already available for real usage:

- [x] No-Cargo packaged install path is validated for tier-1 end-user targets (Linux/macOS/Windows) (evidence: Phase 8 closure + `reports/v1/install_independence.md`)
- [x] Signed distribution trust stack is validated (checksums, signatures, provenance, SBOM) (evidence: `reports/v1/distribution_readiness.md`)
- [x] CLI maturity baseline exists (`vibe --help`, `vibe --version`) with regression gates (evidence: `docs/cli/help_manual.md`, `docs/cli/version_output.md`, workflow `.github/workflows/v1-cli-ux.yml`)
- [x] Self-host transition foundation (M1-M4) is wired with blocking gates and rollback control (evidence: Phase 9 closure + `reports/v1/selfhost_readiness.md`)

What still blocks "full-fledged language for mainstream software development":

- [x] Native runtime contract enforcement gap is closed via native `@require/@ensure` lowering + blocking gate `contract_runtime_enforcement_gate` (evidence: `reports/v1/readiness_dashboard.md`, `reports/v1/smoke_validation.md`, `crates/vibe_cli/tests/contract_runtime_enforcement.rs`)
- [x] Memory/leak and GC-observable lanes are in the default release cycle via `memory_gc_default_gate` (evidence: `.github/workflows/v1-release-gates.yml`, `reports/v1/release_candidate_checklist.md`, `reports/v1/smoke_validation.md`)
- [ ] Dynamic container implementation is still in freeze scope, not full generic container surface (evidence gap: deferred items in `docs/spec/containers.md`)
- [ ] Async/await and explicit thread model are specified but not fully implemented end-to-end (evidence gap: `reports/v1/spec_readiness.md` deferred items)
- [x] Toolchain reproducibility and performance guardrails are hardened (`--locked`, pinned toolchain, clean/no-op/incremental thresholds, bit-identical rebuild gate) (evidence: `.github/workflows/v1-release-gates.yml`, `.github/workflows/v1-packaged-release.yml`, `reports/v1/quality_budgets.json`)
- [ ] Book/docs quality program remains incomplete for mainstream onboarding (evidence gap: section `7.4` + Phase 7 exit item 4)

---

## Phase 10: GA Runtime and Determinism Hardening

Goal: close remaining P0/P1 engineering blockers so release candidates can be promoted without caveats.

### 10.1 Runtime Safety and Contract Enforcement

- [x] Implement contract checks (`@require/@ensure`) in native execution path for `vibe run` and produced binaries (not only example-runner path)
- [x] Add negative/positive runtime contract enforcement suites in CI with blocking status
- [x] Close `V1-P0-CRUNTIME` in `reports/v1/readiness_dashboard.md` with linked workflow evidence
- [x] Add release-gate summary artifact proving no open P0 runtime safety exceptions for candidate

### 10.2 Reproducibility and Determinism Baseline

- [x] Implement true `--locked` mode in CLI build/run/test pathways and enforce in release workflows
- [x] Pin release toolchain version (non-moving) and publish toolchain hash evidence per RC
- [x] Normalize debug/repro metadata (machine/path-stable artifact outputs)
- [x] Add bit-identical rebuild checks across clean runner re-execution for same commit + toolchain

### 10.3 Performance and Reliability Guardrails

- [x] Replace coarse compile-time checks with clean/no-op/incremental benchmark suite
- [x] Expose incremental cache hit/miss telemetry to CLI and CI reports
- [x] Gate compile/runtime regressions with explicit numeric thresholds (latency/memory)
- [x] Run memory/leak + GC-observable lanes in default release candidate cycle (not optional)

### Phase 10 Exit Criteria

- [x] Runtime contract enforcement is blocking and no longer partial (evidence: `v1-release-gates.yml` job `contract_runtime_enforcement_gate`, `reports/v1/readiness_dashboard.md`)
- [x] Reproducible build mode is enforced and evidence-published for RC cycle (evidence: `v1-release-gates.yml` job `bit_identical_rebuild_gate`, `v1-packaged-release.yml` toolchain evidence artifacts)
- [x] Performance and memory guardrails are blocking with thresholds and trend artifacts (evidence: `reports/phase6/metrics/phase6_metrics.json`, `reports/v1/quality_budgets.json`, `v1-release-gates.yml` jobs `metrics_threshold_smoke` + `memory_gc_default_gate`)

---

## Phase 11: Full Language Surface for General-Purpose Programs

Goal: expand beyond the current freeze scope so users can author broader real-world applications without language-surface gaps.

### 11.1 Containers and Text Completeness

- [x] Implement generic `List<T>` / `Map<K,V>` lowering beyond current concrete freeze combinations
- [x] Add container iterators and native `for in` lowering with deterministic semantics
- [x] Add `Str` indexing/slicing APIs and Unicode-safe behavior tests
- [x] Add container equality/hash coverage and deterministic map behavior tests for broader key/value types

### 11.2 Async/Await and Thread Model Implementation

- [x] Land parser/HIR/MIR/runtime implementation for `async` / `await`
- [x] Implement explicit `thread` boundary model and sendability checks across async/thread boundaries
- [x] Add cancellation/timeout behavior tests and deterministic failure propagation checks
- [x] Gate async/thread conformance in `v1-release-gates.yml`

### 11.3 Module and Program-Scale Composition

- [x] Strengthen module/import/package-boundary behavior for multi-module applications
- [x] Add visibility, cyclic-import, and package-layout diagnostics for large codebases
- [x] Add project templates for service/CLI/library layouts with integration tests
- [x] Publish migration and compatibility guidance for stable module evolution

### Phase 11 Exit Criteria

- [x] Real application patterns requiring generic collections, iteration, and text slicing are supported with deterministic tests
- [x] Async/thread semantics move from spec-only to release-gated implementation (evidence: tests + CI gates + docs)
- [x] Multi-module application composition is stable and documented for production teams

---

## Phase 12: Stdlib and Ecosystem Expansion

Goal: provide enough batteries-included surface and package lifecycle tooling for day-to-day software delivery.

### 12.1 Core Standard Library Surface

- [x] Expand standard library modules for filesystem/path, time/duration, serialization (`json`), and HTTP/client-server essentials
- [x] Define stability tiers for stdlib APIs (stable/preview/experimental)
- [x] Add deterministic behavior and error-model tests per stdlib module
- [x] Publish stdlib reference documentation and versioning guarantees

### 12.2 Package Lifecycle and Registry Readiness

- [x] Mature `vibe pkg` for lock/resolve/install parity on real dependency graphs
- [x] Add package publishing flow, registry index format, and provenance/security policy
- [x] Add dependency vulnerability/license policy checks in CI
- [x] Add semver compatibility checks and upgrade assistant docs

### 12.3 Testing and QA Ecosystem

- [x] Expand `vibe test` ergonomics for large test suites (filtering, sharding, richer reporting)
- [x] Add coverage reporting for language/runtime/stdlib/package-manager surfaces
- [x] Add golden snapshot update tooling and policy for stable diagnostics
- [x] Publish ecosystem readiness report for package + stdlib health

### Phase 12 Exit Criteria

- [x] Teams can build typical backend/service/automation applications using documented stdlib APIs without core gaps
- [x] Package resolution/install/publish lifecycle is repeatable, secure, and release-gated
- [x] QA workflow supports project-scale testing and coverage visibility

---

## Phase 13: Developer Experience and Production Operations

Goal: make VibeLang practical as an everyday engineering platform across local dev, CI/CD, incident response, and long-term maintenance.

### 13.1 IDE and Workflow Maturity

- [x] Upgrade LSP from diagnostics-focused to productivity-focused (completion, symbol nav, rename, code actions, formatting integration) (evidence: `crates/vibe_lsp/src/protocol.rs`, `crates/vibe_lsp/src/handlers.rs`, `crates/vibe_lsp/src/session.rs`, `reports/phase13/lsp_productivity_features.md`)
- [x] Add large-workspace performance benchmarks for indexer/LSP paths (evidence: `tooling/phase13/benchmark_editor_ux.py`, `reports/phase13/editor_ux_metrics.json`, `reports/phase13/large_workspace_performance.md`)
- [x] Add editor/CI consistency checks to prevent local-vs-CI behavior drift (evidence: `tooling/phase13/check_diagnostics_parity.py`, `reports/phase13/editor_ci_consistency.json`, `reports/phase13/editor_ci_consistency.md`)
- [x] Publish IDE setup guides and recommended workflows for major editors (evidence: `docs/ide/vscode_cursor_setup.md`, `docs/ide/editor_ci_consistency.md`, `docs/ide/syntax_color_guide.md`, `.github/workflows/phase13-editor-ux.yml`)

### 13.2 Debugging, Profiling, and Observability

- [ ] Define debugging/profiling workflow for Vibe programs (symbols, stack traces, perf diagnostics)
- [ ] Add runtime observability primitives (structured logs/metrics/traces contracts)
- [ ] Add production incident triage playbook specific to Vibe runtime failures
- [ ] Add deterministic crash repro artifact format for bug reports

### 13.3 Release Operations and Lifecycle Governance

- [ ] Define LTS/support windows and compatibility guarantees for v1.x
- [ ] Add security response/CVE handling workflow and disclosure policy
- [ ] Add release-notes automation with known-limitations + breaking-change sections
- [ ] Close Phase 7.4 docs/book program with executable snippet validation and publish docs quality report

### Phase 13 Exit Criteria

- [ ] Developer workflow (edit/build/test/debug/profile) is stable for medium/large projects with measurable SLAs
- [ ] Operational governance (security, support windows, incident response) is documented and exercised
- [ ] Documentation/book program is complete enough for independent onboarding of new teams

---

## Phase 14: Production Adoption and GA Confidence

Goal: prove VibeLang can be used like mainstream languages for sustained software development in real projects.

### 14.1 Pilot Application Program

- [ ] Ship at least two non-trivial reference applications maintained in VibeLang (service + CLI/tooling class)
- [ ] Track defect rate, performance regressions, and developer productivity metrics across pilot cycles
- [ ] Capture migration pain points and convert into language/tooling/docs backlog with owners
- [ ] Publish case-study style evidence reports per pilot

### 14.2 GA Promotion Gate

- [ ] Run consecutive hosted RC cycles with zero open P0 and approved P1 exceptions only
- [ ] Confirm all Phase 10-13 exit criteria are checked with linked evidence artifacts
- [ ] Freeze v1 GA release branch with signed trust bundle + reproducibility + selfhost transition evidence
- [ ] Publish GA readiness announcement report with explicit support/limitations matrix

### Phase 14 Exit Criteria

- [ ] VibeLang is validated for sustained software development in real project scenarios (evidence: pilot reports + RC evidence)
- [ ] GA decision can be made without unresolved production-critical caveats
