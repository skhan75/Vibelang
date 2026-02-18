# VibeLang Development Checklist (v1 Roadmap)

This checklist is the working tracker for building VibeLang from current docs into a production-ready language ecosystem.

Use rules:

- Mark a task complete only when code/tests/docs are merged and runnable.
- Keep checkboxes small and verifiable.
- If scope changes, update this checklist before implementation.

## Critical Design Guardrails (Non-Negotiable)

### Determinism First

- [ ] Reproducible build mode (`--locked`, pinned toolchain, normalized artifacts)
- [ ] Determinism tests: same source + same toolchain => bit-identical output
- [ ] AI sidecar proven non-blocking for parse/type/codegen/link paths
- [ ] Deterministic diagnostics ordering in compiler output

### Safety Defaults

- [ ] Data-race safety strategy documented and enforced in language/runtime behavior
- [ ] Memory safety defaults documented for user code paths
- [ ] Concurrency primitives (`go`, `chan`, `select`) validated with stress tests
- [ ] Contract checks (`@require/@ensure`) active in dev/test profiles by default

### Escape Hatches (Isolated and Auditable)

- [ ] Define unsafe/low-level escape hatch syntax and scope boundaries
- [ ] Require explicit annotation/review path for unsafe blocks
- [ ] Emit audit report listing all unsafe blocks per build

### Transparent Performance Model

- [ ] Allocation visibility available in diagnostics/profile outputs
- [ ] Effect declarations (`@effect`) checked against observed behavior
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

- [x] `vibe check` validates sample files end-to-end
- [x] Typed HIR generated for core language constructs
- [x] Frontend test suite stable in CI
- [x] Deterministic diagnostics and HIR output confirmed by repeat-run tests

---

## Phase 2: Native Backend + Small Stdlib + CLI Tooling

Goal: compile typed IR into native binaries with a minimal standard library and stable CLI workflow.

### 2.1 Codegen and Linking

- [x] Codegen strategy documented (`compiler/codegen/README.md`)
- [x] IR staging documented (`compiler/ir/overview.md`)
- [x] Implement MIR -> backend lowering (Cranelift-first)
- [x] Emit object files for Linux x86_64 first target
- [x] Integrate linker for executable output
- [ ] Add debug info emission basics

### 2.2 Minimal Standard Library

- [x] Define Phase 2 stdlib boundaries with initial `io` contract (`print/println`)
- [x] Implement hello-world IO intrinsic path (`print`/`println` -> runtime)
- [ ] Add deterministic utility APIs for contract/example execution
- [ ] Document stdlib stability policy

### 2.3 CLI Tooling

- [x] Implement `vibe build`
- [x] Implement `vibe run`
- [ ] Implement `vibe test`
- [x] Implement `vibe check`
- [x] Add profile/target flags (`--profile`, `--target`, `--offline`)

### Phase 2 Exit Criteria

- [x] Hello-world style programs compile and run natively
- [ ] Sample specs compile with basic stdlib dependencies
- [x] CLI commands stable for local developer workflows

---

## Phase 3: Ownership/Effect Checker + Concurrency Model

Goal: enforce safe concurrent behavior and effect correctness while preserving ergonomic syntax and automatic GC.

### 3.1 Ownership and Effect Semantics

- [ ] Define ownership/aliasing rules for shared mutable state (thread-safety focused)
- [ ] Implement effect inference and checking against `@effect`
- [ ] Add transitive effect propagation over call graph
- [ ] Add compiler diagnostics for missing/incorrect effects

### 3.2 Concurrency Runtime Model

- [x] Concurrency design drafted (`runtime/concurrency/design.md`)
- [ ] Implement task scheduler (M:N work stealing baseline)
- [ ] Implement typed bounded channels
- [ ] Implement `select` semantics with fairness policy
- [ ] Implement cancellation primitives and propagation

### 3.3 Reliability and Safety Validation

- [ ] Concurrency stress tests (deadlock/contention/cancellation paths)
- [ ] Race-pattern tests and static/dynamic checks where feasible
- [ ] Panic/failure propagation rules validated in task scopes

### Phase 3 Exit Criteria

- [ ] Concurrency primitives stable under stress workloads
- [ ] Effect checker catches mismatches with high signal
- [ ] Runtime behavior aligns with safety-default guardrails

---

## Phase 4: Incremental Indexer + LSP Diagnostics

Goal: provide fast, local-first IDE feedback with semantic understanding and incremental updates.

### 4.1 Semantic Index Core

- [x] Indexer architecture drafted (`compiler/indexer/README.md`)
- [ ] Implement symbol/reference index storage
- [ ] Implement contract/effect metadata indexing
- [ ] Implement file-change incremental update pipeline
- [ ] Persist/reload index safely with schema versioning

### 4.2 LSP and Editor Integration

- [ ] Go-to-definition/references support
- [ ] Real-time diagnostics streaming
- [ ] Intent/contract metadata surfacing in editor UI
- [ ] Performance and stability testing in medium-size projects

### 4.3 Performance Targets

- [ ] Cold index target validated on reference hardware
- [ ] Single-file edit update latency target validated
- [ ] Index memory overhead target validated

### Phase 4 Exit Criteria

- [ ] Local IDE workflow is fast, stable, and offline-capable
- [ ] Semantic index supports downstream AI sidecar reads

---

## Phase 5: AI Intent Engine + Verifier-Gated Suggestions

Goal: add AI productivity features without compromising determinism, cost, or trust.

### 5.1 AI Sidecar Core

- [x] Sidecar architecture drafted (`ai/sidecar/architecture.md`)
- [x] Cost model drafted (`ai/sidecar/cost_model.md`)
- [x] Offline mode drafted (`ai/sidecar/offline_mode.md`)
- [ ] Implement sidecar service with local-first execution
- [ ] Integrate read-only semantic index access

### 5.2 Intent Lint and Suggestions

- [ ] Implement on-demand intent lint command (`vibe lint --intent`)
- [ ] Implement changed-only mode (`vibe lint --intent --changed`)
- [ ] Add confidence + evidence in AI diagnostics output
- [ ] Ensure suggestions are verifier-gated and compiler-revalidated

### 5.3 Risk Controls

- [ ] Enforce latency and cost budgets in runtime policy
- [ ] Enforce non-blocking compile pipeline under AI failure/timeouts
- [ ] Add policy controls for local-only / hybrid / cloud modes
- [ ] Add telemetry dashboards (opt-in only)

### Phase 5 Exit Criteria

- [ ] AI features clearly improve workflow without compile dependency
- [ ] Cost/latency budgets consistently respected
- [ ] Intent lint trusted as advisory signal with low false positives

---

## Phase 6: Self-Hosting Path + Ecosystem

Goal: move from core compiler/runtime to a sustainable developer ecosystem.

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

### Phase 6 Exit Criteria

- [ ] Teams can build, test, publish, and maintain VibeLang projects end-to-end
- [ ] Self-hosting path is demonstrated or scheduled with validated milestones

---

## Cross-Phase Tracking Metrics

- [ ] Compile performance dashboard (clean/no-op/incremental)
- [ ] Runtime performance dashboard (GC pauses, throughput, latency)
- [ ] Contract coverage and failure signal quality
- [ ] Intent lint precision/recall (where measurable)
- [ ] Developer productivity indicators (time-to-first-binary, median feedback loop)

## Current Status Snapshot

- Core strategy/spec docs: drafted and phase-1 ambiguities resolved
- Implementation code: Phase 2 native path delivered for Linux x86_64 (HIR bodies, MIR, Cranelift object emission, runtime/link, `vibe build`/`vibe run`)
- Verification: hello-world compile/run works end-to-end with deterministic build smoke coverage and CI gates
- Next execution focus: broaden backend coverage (additional control-flow/codegen subset, debug info, arm64) and deepen stdlib/runtime capabilities
