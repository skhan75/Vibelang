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
- [ ] AI sidecar proven non-blocking for parse/type/codegen/link paths
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
- [x] Implement backend support for documented core forms used in official specs/examples (`List/Map` paths, member access, method-call lowering)
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

- [ ] Define fixture taxonomy and naming convention for progression levels (`basic`, `intermediate`, `advanced`, `stress`) under `compiler/tests/fixtures/phase7/`
- [ ] Add minimal syntax/lexing fixtures: literals, comments, whitespace, indentation boundaries, unary/binary operators, grouping
- [ ] Add identifier fixtures: valid identifiers, reserved keyword rejection, shadowing behavior, naming edge cases
- [ ] Add parser-recovery fixtures: malformed blocks/annotations with multi-error stability expectations
- [ ] Add type-check fixtures: inference boundaries, mismatch diagnostics, unknown symbol/function handling, deterministic error ordering

#### 7.1.b Annotation/Contract/Intent Coverage

- [ ] Add dedicated fixtures for `@intent`, `@examples`, `@require`, `@ensure`, `@effect` individually and in valid combinations
- [ ] Add invalid-annotation fixtures (unknown tags, malformed payloads, wrong placement) with stable diagnostics
- [ ] Add runtime contract policy tests covering dev/test default behavior and explicit overrides
- [ ] Add `@examples` correctness tests for pass/fail reporting quality (function, input, expected/actual, source span)
- [ ] Add effect conformance tests proving `@effect` declarations align with observed behavior

#### 7.1.c Single-Threaded Program Suite

- [ ] Add canonical single-thread sample programs: hello world, calculator, collection transform pipeline, small state machine
- [ ] Add deterministic output assertions for repeated runs (same input => identical stdout and exit code)
- [ ] Add build artifacts determinism checks for each sample (`.o`, binary hash, debug map stability)
- [ ] Add small "language tour" sample showing functions, control flow, contracts, and examples in one file

#### 7.1.d Multi-Threaded/Concurrency Program Suite

- [ ] Add bounded worker-pool sample using `go`, `chan`, `select`, cancellation token propagation
- [ ] Add fan-in/fan-out sample with fairness assertions on `select`
- [ ] Add timeout/retry sample using `after` branch behavior in `select`
- [ ] Add concurrency stress scenario fixtures with deterministic pass/fail criteria and bounded runtime
- [ ] Add negative tests for concurrency misuse with actionable diagnostics

#### 7.1.e Intent-Driven Development Validation

- [ ] Add intent lint fixtures for "good intent matches implementation" and "intent drift" cases
- [ ] Add changed-only lint mode validation for both git-present and no-git flows
- [ ] Add verifier-gated suggestion tests ensuring rejected suggestions never alter compile determinism
- [ ] Add intent lint quality scoring harness update (`precision`, `recall`, `false-positive trend`) with report output

#### 7.1.f CI and Evidence for Item 1

- [ ] Add dedicated workflow `.github/workflows/phase7-language-validation.yml` for corpus execution
- [ ] Publish report `reports/phase7/language_validation_matrix.md` with pass/fail matrix (feature x test level)
- [ ] Publish report `reports/phase7/sample_programs_catalog.md` with run/build/test commands and expected outputs

### 7.2 Ordered Item 2 — GitHub README (Product-Grade)

#### 7.2.a README Content and Positioning

- [ ] Rewrite root `README.md` with product-quality structure and clear sections
- [ ] Add concise project pitch: what VibeLang is and why it was built
- [ ] Add "What VibeLang solves today" section with concrete, current capabilities
- [ ] Add "What is experimental / in-progress" section to set user expectations honestly
- [ ] Add use-case section (systems tooling, concurrent services, deterministic build pipelines, intent-aware development)

#### 7.2.b Quickstart and Installation UX

- [ ] Add installation paths: from source, local binary usage, and future packaged release placeholder
- [ ] Add 60-second quickstart (`vibe new`, `vibe run`, `vibe test`, `vibe fmt`, `vibe doc`)
- [ ] Add hello-world + one contract/intent sample snippet
- [ ] Add troubleshooting section (common setup/compiler toolchain issues)

#### 7.2.c Visual and Navigation Quality

- [ ] Add section anchors/table of contents for easy scanning
- [ ] Add architecture diagram (compiler core, runtime, indexer/lsp, sidecar boundaries)
- [ ] Add roadmap snapshot links to `docs/development_checklist.md` and phase reports
- [ ] Add contribution/start-here section (build, test, lint, CI expectations)

#### 7.2.d README Validation

- [ ] Add markdown lint/link check for README in CI
- [ ] Add quickstart smoke check in CI to ensure commands in README stay executable

### 7.3 Ordered Item 3 — Tightened V1 Production Release Checklist

#### 7.3.a Scope Freeze and Release Gates

- [ ] Define explicit v1 feature scope freeze list and non-goals list
- [ ] Convert all remaining top-level unchecked guardrails into owned release gates (owner/severity/target milestone)
- [ ] Add release blocker policy (`P0`/`P1` criteria) and merge gate alignment
- [ ] Create v1 release readiness dashboard report (`reports/v1/readiness_dashboard.md`)

#### 7.3.b Engineering Quality Hardening

- [ ] Close remaining determinism/safety/performance unchecked items in this checklist with evidence
- [ ] Define minimum test coverage expectations for parser/type/runtime/cli/intent-lint paths
- [ ] Add long-run stability/soak tests with bounded budgets and pass thresholds
- [ ] Add packaging integrity checks (checksums/signatures/provenance plan)
- [ ] Add upgrade/downgrade compatibility test path between adjacent v1.x versions

#### 7.3.c Operational Readiness

- [ ] Define release candidate process (`rc1`, `rc2`, promote/reject criteria)
- [ ] Define rollback playbook for bad release detection and mitigation
- [ ] Define issue triage SLA and bug severity taxonomy for public users
- [ ] Define telemetry/privacy statement for optional AI-related signals
- [ ] Add "known limitations" publication gate before each release

#### 7.3.d CI/Reporting for V1 Tightening

- [ ] Add workflow `.github/workflows/v1-release-gates.yml` for consolidated v1 blocking checks
- [ ] Publish `reports/v1/release_candidate_checklist.md` template and first run
- [ ] Require all v1 gate reports linked in release PR description

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

- [ ] Item 1 completed: comprehensive language validation matrix and sample program catalog are green with reproducible evidence (`workflow .github/workflows/phase7-language-validation.yml`, reports under `reports/phase7/`)
- [ ] Item 2 completed: README is public-ready, accurate, and CI-validated against command drift (`README.md`, README smoke workflow job)
- [ ] Item 3 completed: v1 production release gates are explicitly defined, owned, and passing for at least one release-candidate cycle (`workflow .github/workflows/v1-release-gates.yml`, `reports/v1/`)
- [ ] Item 4 completed: book/docs program is structured, CI-gated, and includes tested chapter examples across core language/tooling surfaces (`book/`, docs CI jobs, `reports/docs/documentation_quality.md`)
