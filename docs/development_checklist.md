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

- [ ] Add dual-extension source discovery support (`.vibe` + `.yb`) in CLI/indexer/LSP/watcher pipelines
- [ ] Update changed-file detection and workspace scans to include both extensions (git diff globs, recursive collectors, incremental hashing)
- [ ] Add compatibility test matrix proving command parity for both extensions (`check`, `build`, `run`, `test`, `lint`, `index`, `lsp`)
- [ ] Make `vibe new` and docs/spec samples default to `.yb` while preserving `.vibe` backward compatibility
- [ ] Publish extension migration guide with deprecation timeline and opt-in warning policy
- [ ] Define `.vibe` removal gate using adoption + CI parity thresholds (no hard removal before thresholds are met)

### 6.1 Self-Hosting Roadmap

- [ ] Define bootstrap strategy (host language -> VibeLang compiler transition)
- [ ] Build milestone plan for partial then full self-hosting
- [ ] Establish conformance tests to compare bootstrap vs self-host behavior

### 6.2 Package and Build Ecosystem

- [ ] Package manager design and lockfile format
- [ ] Dependency resolution and reproducible install
- [ ] Registry/mirror strategy (including offline workflows)

### 6.3 Developer Experience Tooling

- [ ] Formatter (`vibe fmt`)
- [ ] Docs generator (`vibe doc`)
- [ ] Unified test runner with contract/example integration
- [ ] Project scaffolding command (`vibe new`)

### 6.4 Adoption and Stability

- [ ] Versioning and compatibility policy
- [ ] Release pipeline and changelog process
- [ ] Migration guides between language/toolchain versions
- [ ] Define source-extension compatibility policy (`.vibe` legacy, `.yb` canonical) for v1.x

### 6.5 Portability and Conformance Governance

- [ ] Expand native target support toward charter targets (Linux arm64, macOS arm64) with parity checklist
- [ ] Add cross-target CI matrix for build/run determinism and runtime smoke tests
- [ ] Publish target-tier support matrix (feature coverage, performance expectations, known limitations)
- [ ] Require phase-complete evidence: spec-to-runtime conformance tests + CI gates before marking a phase done
- [ ] Maintain a release-visible limitations/debt register with owner, severity, and target phase

### Phase 6 Exit Criteria

- [ ] Teams can build, test, publish, and maintain VibeLang projects end-to-end (evidence: release pipeline dry-run report + package manager e2e CI)
- [ ] Self-hosting path is demonstrated or scheduled with validated milestones (evidence: bootstrap-vs-self-host conformance report in `reports/phase6/self_hosting_conformance.md`)
- [ ] Target-tier support and conformance governance process are operational in release workflow (evidence: CI matrix results + published support matrix)
- [ ] `.yb` is canonical for new projects while `.vibe` remains regression-free during migration (evidence: dual-extension parity CI + report `reports/phase6/source_extension_migration.md`)

---

## Cross-Phase Tracking Metrics

- [ ] Compile performance dashboard (clean/no-op/incremental)
- [ ] Runtime performance dashboard (GC pauses, throughput, latency)
- [ ] Contract coverage and failure signal quality
- [ ] Intent lint precision/recall (where measurable)
- [ ] Developer productivity indicators (time-to-first-binary, median feedback loop)
- [ ] Spec conformance dashboard (documented constructs vs runtime-validated coverage)
- [ ] Unsupported-feature backlog trend (count, severity, time-to-resolution)
- [ ] Cross-target compatibility pass rate by profile/target
- [ ] Source-extension adoption ratio (`.yb` vs `.vibe`) and dual-support parity pass rate

## Current Status Snapshot

- Core strategy/spec docs: drafted and phase-1 ambiguities resolved
- Implementation code: Phase 5 conformance hardening and AI sidecar baseline delivered (`vibe_sidecar`, `vibe lint --intent`, verifier-gated suggestions, policy controls, telemetry and evidence bundle)
- Verification: phase5 sidecar/lint integration + native conformance suites + compile parity and budget guard checks are green (workflows `.github/workflows/phase1-frontend.yml` through `.github/workflows/phase5-ai-sidecar.yml`)
- Next execution focus: Phase 6 self-hosting path and ecosystem scale-out, starting with phased `.vibe` -> `.yb` migration (dual-support first), then package manager/formatter/docs/test UX and target-matrix governance
