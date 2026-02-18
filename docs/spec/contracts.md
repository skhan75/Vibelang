# VibeLang Contracts Spec (v0.1 Draft)

## Purpose

Contracts make expected behavior executable and verifiable at compile/test time.

They are designed to be:

- Easy to write next to implementation
- Deterministic to evaluate
- Cheap to run locally

## Contract Annotations

V0.1 supports:

- `@intent "text"`
- `@examples { case* }`
- `@require predicate`
- `@ensure predicate`
- `@effect effect_name`

Effect names are frozen for Phase 1 in `docs/spec/phase1_resolved_decisions.md` and `docs/spec/grammar_v0_1.ebnf`.

## Placement Rules

- Contracts must appear at the top of function body, before executable statements.
- Multiple annotations of same kind are allowed (except `@intent` currently recommended as one line).

## v0.1 Conciseness Guardrails

To avoid contract noise and keep code readable:

- Prefer one `@intent` line per function.
- Keep `@examples` focused on high-signal cases (happy path, boundary, empty/zero).
- Prefer a small number of precise `@ensure` predicates over many overlapping checks.
- Use `@effect` tags only for side effects that materially aid reasoning/review.

These are style guardrails in v0.1 (linted), not hard parser limits.

## `@intent`

- Human-readable objective for the function.
- Stored in semantic index metadata.
- Not executed directly as code.
- Can be consumed by AI sidecar, docs generator, and review tools.

Example:

```txt
@intent "k largest numbers, sorted desc"
```

## `@examples`

Defines executable input/output examples.

Syntax:

```txt
@examples {
  f(a1, a2) => expected
  f(b1, b2) => expected2
}
```

Rules:

- Left side must be call expression to current function or a referenced function under test context.
- Right side must be deterministic expression.
- Compiler lowers examples into generated test cases.

## `@require`

Precondition checked at function entry.

Example:

```txt
@require k >= 0
```

Semantics:

- If false, contract violation path is executed.
- In tests/dev this is a hard failure with source span.
- Release behavior is policy-driven (panic/error return) and configurable.

## `@ensure`

Postcondition checked before function return.

Special forms:

- `.` stands for return value.
- `old(expr)` snapshots entry-time expression.

Example:

```txt
@ensure len(.) == min(k, len(xs))
@ensure from.balance == old(from.balance) - amount
```

## `@effect`

Declares permitted/expected side effects.

Initial effect vocabulary:

- `alloc`
- `mut_state`
- `io`
- `net`
- `concurrency`
- `nondet`

Semantics:

- Compiler builds effect summary for each function.
- Summary is checked against declared effects.
- Transitive effect propagation is supported for call graphs.

## Lowering Strategy

Contracts lower into typed intermediate checks:

1. Parse contract annotations into contract AST nodes.
2. Resolve names/types in same pass as function body.
3. Insert synthetic pre/post check nodes in HIR.
4. Emit generated tests from `@examples` into test harness IR.
5. Run checks under selected profile policy.

## Determinism Rules

To keep checks reproducible:

- Contract expressions cannot call `@effect nondet` functions unless explicitly allowed.
- Contract evaluation cannot perform I/O.
- Time and randomness are unavailable in contract expressions.

## Auditability Requirements

Checks must be easy to inspect and trust:

- Compiler must preserve source spans for each lowered check.
- Generated checks/tests must be reproducible from source + toolchain version.
- Tooling should support "explain check lowering" output for debugging and review.
- AI-generated contract suggestions must pass the same deterministic checker as handwritten contracts.

## Error Messages

Contract violation diagnostics should include:

- Function name
- Failed contract text
- Bound values used in evaluation
- Source span

Example format:

```txt
Contract violation in topK(xs, k):
  failed: len(.) == min(k, len(xs))
  values: len(.)=3, k=2, len(xs)=5
  at app/math.vibe:18:3
```

## Future Expansion (Post-v1)

- Quantifiers (`forall`, `exists`) with bounded collections
- Property generators for auto-fuzz beyond hand-written examples
- SMT-backed proving for selected contract subsets
