# Chapter 3: Types, Functions, and Errors

This chapter covers VibeLang’s type model, function design conventions, and
error semantics. These areas form the backbone of correctness in both hand-written
and AI-assisted code.

## 3.1 Type System Goals

VibeLang’s type system is designed for:

- static safety with low authoring noise,
- deterministic type checking,
- explicit behavior at API and concurrency boundaries.

Type checking should be stable and reproducible: the same code should not
"mysteriously infer" different meanings across runs.

## 3.2 Type Categories

### Primitive

- `Bool`
- `Str`
- signed integers: `i8`, `i16`, `i32`, `i64`, `isize`
- unsigned integers: `u8`, `u16`, `u32`, `u64`, `usize`
- floating point: `f32`, `f64`

### Compound

- `List<T>`
- `Map<K,V>`
- `Result<T,E>`
- user-defined `type` declarations
- `Chan<T>`

### Optional

- canonical optional syntax: `T?`
- empty optional value: `none`

## 3.3 Inference and Annotation Strategy

VibeLang allows local inference:

```txt
count := 10
name := "vibe"
```

But for public APIs, explicit signatures are strongly preferred:

```txt
pub parsePort(raw: Str) -> Result<u16, ParseError> {
  ...
}
```

A practical rule for teams:

- infer in local glue code,
- annotate at module boundaries and shared libraries.

This keeps internal ergonomics high without sacrificing public clarity.

## 3.4 Assignability and Conversions

Assignability rules are intentionally narrow:

- identical types are assignable,
- selected numeric widening may be allowed,
- `none` is assignable to optional `T?`,
- potentially lossy conversions must be explicit.

Do not assume C-style or dynamic-language coercions. VibeLang prefers explicit
conversion decisions where correctness can be affected.

## 3.5 Numeric Defaults and Pitfalls

From the numeric model:

- integer literals default to `i64`,
- float literals default to `f64`,
- suffixes are authoritative (`42u32`, `0.5f32`).

Examples:

```txt
count: u32 := 10u32
small: i8 := 12i8
ratio: f32 := 0.5f32
pi := 3.1415926535
```

Important pitfall: mixed signed/unsigned arithmetic often requires explicit
conversions. Keep conversions near the operation so reviewers can verify intent.

## 3.6 Functions: Shape and Semantics

Function calls evaluate arguments left-to-right. Return paths must remain type
compatible.

Example:

```txt
pub choose(a: i64, b: i64, pickA: Bool) -> i64 {
  if pickA {
    a
  } else {
    b
  }
}
```

Tail expressions are allowed:

```txt
pub add(a: i64, b: i64) -> i64 {
  a + b
}
```

Explicit return remains valid and can improve clarity in longer functions.

## 3.7 `Result<T,E>` and `?`

Recoverable errors use explicit channels via `Result<T,E>`.

```txt
pub parseNonZero(raw: Str) -> Result<i64, ParseError> {
  n := parse_i64(raw)?
  if n == 0 {
    err(ParseError.zero_not_allowed())
  } else {
    ok(n)
  }
}
```

`?` behavior:

- on success (`ok(v)`), it yields `v`,
- on failure (`err(e)`), it returns early with a compatible error channel.

This keeps propagation explicit and auditable.

## 3.8 Contract Failures vs Result Failures

VibeLang has more than one failure class:

1. `Result` failures (domain-recoverable),
2. contract failures (`@require` / `@ensure`),
3. panic/trap for unrecoverable runtime failures.

Treat them differently in design:

- use `Result` for expected business failures,
- use contracts for invariant enforcement,
- use panic/trap for truly unrecoverable conditions.

## 3.9 Concurrency-Aware Types

Types crossing boundaries (`go`, `thread`, async handoff, channel send) must
satisfy sendability constraints.

That means type design is not isolated from runtime strategy. A type that is
easy to use in synchronous code may need structural adjustment to become safe
for concurrent boundaries.

## 3.10 Practical API Pattern

A robust VibeLang API often follows this shape:

```txt
pub transform(input: Input) -> Result<Output, DomainError> {
  @intent "convert validated input into normalized output"
  @require is_valid(input)
  @ensure is_normalized(.)
  @effect alloc
  @effect io

  ...
}
```

Notice how type channel, contract layer, and effect layer reinforce each other.

## 3.11 Diagnostic Quality Expectations

Type diagnostics should include:

- stable diagnostic code,
- source span,
- expected vs actual type,
- optional fix guidance.

For team workflows, this is critical: stable diagnostics make CI behavior and
editor feedback reliable.

## 3.12 Clarifying Error Semantics in Practice

A common misunderstanding is to treat all failure signals as equivalent. In
VibeLang they are intentionally distinct:

- `Result` represents expected, domain-level failure channels,
- contract failures represent violated assumptions or invariants,
- panic/trap signals unrecoverable execution failure.

This distinction is not academic. It determines caller behavior, retry strategy,
alerting policy, and incident triage speed. A payment API returning
`Result<Receipt, PaymentError>` communicates something fundamentally different
from a contract failure caused by a violated precondition. The first is often a
business condition. The second usually indicates a caller bug or drifted system
assumption.

When writing docs and APIs, always state which failure class is expected and why.
That is one of the highest-leverage clarity improvements you can make.

## 3.13 Chapter Checklist

You should now be able to:

- distinguish primitive/compound/optional types,
- design explicit function signatures for module boundaries,
- use `Result<T,E>` and `?` intentionally,
- avoid accidental coercion assumptions,
- reason about error channels vs contract failures.

---

Next: Chapter 4 covers contracts and executable examples in depth.
