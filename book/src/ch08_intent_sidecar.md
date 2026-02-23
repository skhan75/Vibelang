# Chapter 8: Intent-Driven Development and Sidecar Model

Intent-Driven Development (IDD) is the conceptual center of VibeLang.

This chapter explains:

- what intent means technically,
- how intent relates to contracts and effects,
- where optional AI linting fits,
- how to keep trust boundaries clear.

## 8.1 The Core IDD Loop

A practical IDD loop:

1. declare intent (`@intent`),
2. verify behavior (`@examples`, `@require`, `@ensure`),
3. implement and run,
4. inspect intent drift warnings (`vibe lint --intent`),
5. ship only after deterministic toolchain checks pass.

This loop creates a strong feedback cycle for both humans and AI-generated
patches.

## 8.2 Intent Is Purpose, Not Implementation

Intent should describe expected behavior outcome:

```txt
@intent "charge payment and return approved receipt"
```

Avoid implementation-heavy intent lines:

```txt
@intent "call gateway.charge then map response fields"
```

The first remains stable across refactors; the second drifts quickly.

## 8.3 Intent + Contracts + Effects = Behavioral Envelope

These pieces serve different roles:

- `@intent`: what outcome should hold,
- `@examples`: concrete expected scenarios,
- `@require/@ensure`: formal entry/exit constraints,
- `@effect`: side-effect surface.

Together, they define a function’s behavioral envelope.

## 8.4 Intent Drift: What It Means

Intent drift occurs when implementation no longer matches stated purpose.

Detection signals include:

- contract failures,
- example failures,
- semantic/index deltas,
- optional sidecar warnings.

Drift is especially common in high-velocity AI-assisted refactoring loops.

## 8.5 Optional AI Linting Model

VibeLang’s intent linting model is intentionally optional and non-blocking by
default:

```bash
vibe lint --intent
vibe lint --intent --changed
```

Design principles:

- advisory output with confidence/evidence,
- local-first workflow preference,
- no dependency on AI availability for compiler correctness.

## 8.6 Trust Boundary: Determinism Decides

A critical architectural principle:

> AI can assist authoring and drift analysis.  
> Deterministic compiler/runtime path remains the correctness authority.

This protects teams from model variance while still getting productivity gains.

## 8.7 Authoring High-Signal Intent

High-signal intent checklist:

- one sentence,
- concrete outcome,
- domain language your team already uses,
- no internal implementation details.

For exported functions, pair intent with at least one meaningful example.

## 8.8 Example: Payment Domain IDD

```txt
pub settle(payment: Payment) -> Receipt {
  @intent "charge payment and return approved receipt"
  @examples {
    settle(valid_small_usd_payment()) => approved_small_receipt()
  }
  @require payment.amount > 0
  @require payment.currency == "USD"
  @ensure .status == "approved"
  @effect io

  gateway.charge(payment)
}
```

This function can be reviewed quickly:

- intent is clear,
- assumptions are explicit,
- success shape is explicit,
- side effects are explicit.

## 8.9 Team Policy Patterns

Common policy levels:

- **base mode**: intent lint warnings only,
- **strict branch mode**: require intent lint pass on changed files,
- **critical path mode**: gate only high-confidence drift findings.

Adopt policy gradually; avoid over-gating too early.

## 8.10 Scaling IDD in Large Repos

For larger codebases:

- enforce intent on exported/public APIs first,
- prioritize examples for business-critical functions,
- run `--changed` lint mode in PR loops,
- use index telemetry to keep lint cost bounded.

This gives most of IDD’s value without overwhelming teams.

## 8.11 Anti-Patterns

Avoid:

- writing intent for every tiny private helper regardless of value,
- treating AI lint output as unquestionable truth,
- skipping deterministic tests because "intent lint is green,"
- hiding real side effects by under-declaring `@effect`.

## 8.12 Clarification: Sidecar Findings Are Guidance, Not Compiler Truth

Intent lint output can be highly useful, especially for drift detection in
AI-assisted refactors, but it should be interpreted as advisory analysis. The
core correctness authority remains deterministic compiler/runtime checks,
contracts, and tests.

This distinction is fundamental to VibeLang's trust model. It lets teams gain AI
productivity while preserving deterministic release behavior and auditable
failure boundaries.

## 8.13 Chapter Checklist

You should now be able to:

- write strong intent statements,
- connect intent to contracts and effects,
- run and interpret intent lint output,
- enforce proper trust boundaries between AI and deterministic tooling.

---

Next: Chapter 9 covers migration and compatibility strategy.
