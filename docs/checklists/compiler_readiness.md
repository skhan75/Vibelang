# Compiler Readiness Checklist (Canonical)

Last updated: 2026-03-01

## Purpose

This checklist is the **canonical rollup** for what must be true for the VibeLang compiler/toolchain
to be robust enough that we can safely author **more of VibeLang in VibeLang** (self-hosting
expansion) without sacrificing determinism, diagnostics quality, or release safety.

It intentionally **links to existing canonical checklists and evidence** instead of duplicating
long checklists across multiple files.

## Canonical sources this checklist rolls up

- Language + ecosystem program checklist (phases, existing self-host gates):
  - `docs/checklists/development_checklist.md`
- Feature + stdlib gaps needed for production apps (incl. JSON/HTTP/conversions required for
  real-world self-hosted tooling code):
  - `docs/checklists/features_and_optimizations.md` (see section **F**)
- Release-candidate gates (including self-host RC cycle gates):
  - `docs/checklists/release_candidate_checklist.md`

## Evidence locations (expected)

- Self-host readiness + parity metrics:
  - `reports/v1/selfhost_readiness.md`
  - `reports/v1/selfhost_readiness.json`
  - `reports/v1/selfhost_m2_readiness.md`
  - `reports/v1/selfhost_m3_expansion.md`
- Determinism / repeat-run checks:
  - `tooling/phase12/repeat_run_check.py`
- Crash repro artifacts + debugging workflow:
  - `docs/support/crash_repro_format.md`
  - `docs/debugging/workflow.md`

If any of the above reports are missing, that is itself a readiness failure: either the gate is not
wired, or the evidence is not being produced.

---

## 1) Determinism, repeatability, and “stable outputs” (P0)

These are non-negotiable for self-hosting: the compiler is part of the build system.

- [ ] **Stable diagnostics ordering**: the same input produces byte-for-byte identical diagnostics.
  - **Tracking**: `docs/checklists/development_checklist.md` (determinism items, self-host gates).
  - **Spec hooks**: `docs/spec/type_system.md` (diagnostic requirements), `docs/spec/semantics.md`.
- [ ] **Stable IR output**: when emitting AST/HIR/MIR/snapshots, output is deterministic across runs.
  - **Tracking**: `docs/checklists/development_checklist.md` (IR staging + snapshot determinism).
  - **Spec hooks**: `docs/spec/syntax.md`, `docs/spec/grammar_v1_0.ebnf`.
- [ ] **Repeat-run stability gate**: `vibe check/run/test/fmt` results are stable in a repeat-run lane.
  - **Acceptance**: repeat-run checks pass on representative fixture + example corpus.
- [ ] **No hidden nondeterminism**: randomness, timestamps, env-dependent paths are either removed from
  outputs or explicitly normalized.

**Example scenario (deterministic diagnostics)**

```vibe
// Same file, same toolchain, same flags => identical diagnostic order + text.
pub main() -> Int {
  x := 1
  x = "nope"  // type error
  0
}
```

---

## 2) Compiler correctness and soundness envelope (P0)

- [ ] **Parser correctness**: grammar v1 is implemented with high-signal errors and recovery rules
  that do not cascade into misleading diagnostics.
  - **Spec hooks**: `docs/spec/grammar_v1_0.ebnf`, `docs/spec/syntax.md`.
- [ ] **Type checking soundness baseline**: no known “accept invalid program” bugs in core features
  required for self-hosted components (modules, types, containers, effects, concurrency).
  - **Spec hooks**: `docs/spec/type_system.md`, `docs/spec/containers.md`,
    `docs/spec/ownership_sendability.md`.
- [ ] **Verifier/gates for suggestions**: “help” suggestions are correctness-checked (no bogus fixes).
  - **Tracking**: `docs/checklists/development_checklist.md` (suggestion verifier gate).
- [ ] **IR invariants enforced**: invalid MIR/IR states are rejected with actionable errors (never
  emitted into codegen to crash later).

---

## 3) Robust diagnostics UX (P0)

Self-hosted compiler code will be maintained by humans daily; diagnostics are the UI.

- [ ] **Stable diagnostic codes**: every compiler error has a stable code and consistent structure.
  - **Spec hooks**: `docs/spec/type_system.md` (“Type Errors” requirements).
- [ ] **Spans are correct**: primary span points to the real root cause; secondary spans add context.
- [ ] **Error model consistency**: `Result`, `?`, contract failures, panics/traps align with spec.
  - **Spec hooks**: `docs/spec/error_model.md`, `docs/spec/contracts.md`.
- [ ] **Allocation + unsafe audit surfaces are reliable** (release-gated).
  - **Tracking**: `docs/checklists/development_checklist.md` (unsafe governance + allocation gates).

---

## 4) Crash safety, debugging, and bug-reportability (P0)

- [ ] **No compiler panics on user input** (except deliberate “internal error” paths that emit crash
  repro artifacts).
  - **Acceptance**: “internal error” always emits a reproducible crash report bundle.
- [ ] **Crash repro artifact format is enforced**.
  - **Evidence**: `docs/support/crash_repro_format.md` + CI gate outputs.
- [ ] **Debugging workflow is documented and validated** (symbols, stack traces, perf diagnostics).
  - **Evidence**: `docs/debugging/workflow.md` + CI evidence report.

---

## 5) Self-hosting transition controls (P0)

Self-hosting is not a single switch; it is a controlled migration with fallbacks.

- [ ] **Bootstrap strategy is documented** and matches what CI executes.
  - **Evidence**: `reports/phase6/bootstrap_strategy.md`
- [ ] **Conformance harness exists** comparing host vs self-host behavior for promoted components.
  - **Evidence**: `reports/phase6/self_hosting_conformance.md`
- [ ] **M1/M2/M3/M4 gates are wired and blocking** for release-candidate branches.
  - **Tracking**: `docs/checklists/development_checklist.md` (self-host gates),
    `docs/checklists/release_candidate_checklist.md` (RC gates).
- [ ] **Per-component fallback toggles exist** (instant rollback to host implementation).
  - **Acceptance**: toggles are tested; rollback is documented.
  - **Evidence**: self-host transition playbook + toggle tests.
- [ ] **At least one RC cycle** has shipped with a promoted self-host default component and no parity
  regressions.

---

## 6) “Write more VibeLang in VibeLang” prerequisites (P0 → P1)

Even if the compiler is correct, self-hosting expansion will stall if VibeLang lacks basic libraries.
These are the **minimum** capabilities needed to comfortably author compiler/tooling components in
VibeLang.

- [ ] **Text + bytes utilities** are adequate for compiler workloads (tokenization, formatting).
  - **Tracking**: `docs/checklists/features_and_optimizations.md` `F-06`, `F-07`.
  - **Spec hooks**: `docs/spec/strings_and_text.md`.
- [ ] **Filesystem + path operations** cover compiler needs (read/write, atomic updates, directory
  walks) with explicit effects and good errors.
  - **Tracking**: `docs/checklists/features_and_optimizations.md` (stdlib items) and existing stdlib
    docs under `stdlib/`.
- [ ] **JSON parse/stringify** exists for tool outputs and interoperability (index caches, LSP-ish
  payloads, metadata).
  - **Tracking**: `docs/checklists/features_and_optimizations.md` `F-01`, `F-02`.
- [ ] **Conversions/parsing** are safe and `Result`-based (no sentinel returns).
  - **Tracking**: `docs/checklists/features_and_optimizations.md` `F-05` + language `C-07`.
- [ ] **Logging/telemetry + config** exist so tools aren’t “println debugging”.
  - **Tracking**: `docs/checklists/features_and_optimizations.md` `F-09`, `F-10`.

---

## 7) Exit criteria: “compiler is ready for broad self-hosting”

You can consider the compiler/toolchain robust for broad self-hosting expansion when:

- [ ] All **P0** sections above are checked.
- [ ] M2 and expanded M3 components pass parity gates in **consecutive** CI runs.
- [ ] At least one promoted self-host default component has passed an **RC cycle** with rollback
  controls validated.
- [ ] The stdlib prerequisites in section **6** are sufficiently complete that VibeLang-authored
  compiler tooling code is not forced into workaround patterns.

