# VibeLang Spec Coverage Matrix

Status: release-gated traceability matrix for normative specification rules.

## Legend

- `implemented`: covered by current tests/fixtures/tooling evidence.
- `deferred`: spec is defined but implementation remains intentionally deferred;
  target tracked in checklist/readiness reports.

## Rule Coverage

| Rule ID | Area | Normative Requirement | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- |
| SPEC-SYN-001 | Syntax | v1 grammar is source of truth for production syntax surface | implemented | `docs/spec/grammar_v1_0.ebnf`, `docs/spec/README.md` | Grammar precedence enforced by consistency validator |
| SPEC-SYN-002 | Syntax | `match`, `break`, `continue` are first-class control-flow forms | implemented | `docs/spec/syntax.md`, `docs/spec/control_flow.md`, `docs/spec/grammar_v1_0.ebnf` | Spec-level reconciliation complete |
| SPEC-OPT-001 | Type System | Optional typing uses canonical `T?` syntax and `none` literal | implemented | `docs/spec/syntax.md`, `docs/spec/type_system.md`, `docs/spec/grammar_v1_0.ebnf` | `Option<T>` retained as explanatory alias only |
| SPEC-CON-001 | Contracts | Contracts must appear before executable statements in function body | implemented | `docs/spec/contracts.md`, `docs/spec/syntax.md`, `compiler/tests/fixtures/contract_err/annotation_after_statement.vibe` | Diagnostic behavior validated by frontend fixture tests |
| SPEC-EFF-001 | Effects | Effect vocabulary is fixed and unknown tags are diagnostics errors | implemented | `docs/spec/contracts.md`, `docs/spec/phase1_resolved_decisions.md`, `compiler/tests/fixtures/effect_err/transitive_missing_effect.vibe` | Frozen set: alloc/mut_state/io/net/concurrency/nondet |
| SPEC-TYP-001 | Type System | Static typing with deterministic local inference | implemented | `docs/spec/type_system.md`, `crates/vibe_cli/tests/frontend_fixtures.rs` | Covered by parse/type fixtures and deterministic snapshot tests |
| SPEC-NUM-001 | Numerics | Fixed-width integer/floating families and literal suffixes | implemented | `docs/spec/numeric_model.md`, `docs/spec/grammar_v1_0.ebnf` | Representation and width policy now normative |
| SPEC-NUM-002 | Numerics | Overflow/underflow policy is explicit and profile-aware | deferred | `docs/spec/numeric_model.md`, `docs/development_checklist.md` | deferred: full runtime-policy enforcement tracked under `7.3.f` |
| SPEC-MUT-001 | Mutability | Immutability by default; mutation requires explicit mutability | implemented | `docs/spec/mutability_model.md`, `docs/spec/syntax.md` | Assignment legality model specified |
| SPEC-STR-001 | Strings | `Str` uses UTF-8 with explicit escape/index/slice semantics | implemented | `docs/spec/strings_and_text.md` | Language-level text contract is now explicit |
| SPEC-CNT-001 | Containers | Dynamic `List`/`Map` operations have deterministic semantics | implemented | `docs/spec/containers.md`, `reports/v1/dynamic_containers_conformance.md`, workflow .github/workflows/v1-release-gates.yml job dynamic_containers_gate | Implemented for the `7.3.f.1` freeze scope (`List<Int>`, `Map<Int, Int>`, `Map<Str, Int>`, and deterministic ordering policy) |
| SPEC-CFG-001 | Control Flow | Loop/termination and branch semantics are explicitly defined | implemented | `docs/spec/control_flow.md`, `docs/spec/grammar_v1_0.ebnf` | Includes break/continue and match behavior |
| SPEC-CONC-001 | Concurrency | `go`/channel/select scheduler semantics are documented | implemented | `docs/spec/concurrency_and_scheduling.md`, `crates/vibe_cli/tests/phase7_concurrency.rs`, `crates/vibe_runtime/src/lib.rs` | Runtime smoke and stress tests provide evidence |
| SPEC-ASY-001 | Async | `async`/`await` semantics are specified for v1 target | deferred | `docs/spec/async_await_and_threads.md`, `docs/development_checklist.md` | deferred: parser/runtime implementation not yet fully landed |
| SPEC-THR-001 | Threads | Explicit `thread` boundary semantics are specified | deferred | `docs/spec/async_await_and_threads.md`, `docs/development_checklist.md` | deferred: implementation targeted after core runtime milestones |
| SPEC-OWN-001 | Ownership | Cross-boundary sendability (`go`/async/thread/channel) is specified | implemented | `docs/spec/ownership_sendability.md`, `compiler/tests/fixtures/ownership_err/unknown_sendability_in_go.yb` | Ownership diagnostics already exercised in fixture tests |
| SPEC-MEM-001 | Memory Model | Happens-before and synchronization boundaries are normative | implemented | `docs/spec/memory_model_and_gc.md`, `docs/spec/concurrency_and_scheduling.md` | Language-level memory visibility contract published |
| SPEC-GC-001 | GC | GC correctness and observability contracts are documented | deferred | `docs/spec/memory_model_and_gc.md`, `crates/vibe_cli/tests/phase7_v1_tightening.rs` | deferred: GC-observable lane still feature-gated |
| SPEC-ERR-001 | Error Model | `Result`/`?`/contract-failure semantics are explicit | implemented | `docs/spec/error_model.md`, `docs/spec/contracts.md`, `crates/vibe_cli/tests/phase2_native.rs` | Includes profile-dependent contract behavior |
| SPEC-MOD-001 | Modules | Deterministic module/import/visibility rules are defined | implemented | `docs/spec/module_and_visibility.md`, `docs/spec/syntax.md` | Locked-mode and import determinism documented |
| SPEC-ABI-001 | ABI/FFI | ABI and interop boundaries are documented | implemented | `docs/spec/abi_and_ffi.md`, `docs/spec/numeric_model.md` | Target and layout constraints defined |
