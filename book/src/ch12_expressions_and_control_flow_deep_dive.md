# Chapter 12: Expressions and Control-Flow Deep Dive

This chapter expands on expression behavior and loop/control semantics with a
focus on correctness, readability, and deterministic behavior.

## 12.1 Expression-Oriented Style

VibeLang supports expression-oriented function bodies. A function can return the
final expression value without explicit `return`.

```txt
pub max2(a: i64, b: i64) -> i64 {
  if a >= b { a } else { b }
}
```

Use explicit `return` where early exits improve clarity.

## 12.2 Evaluation Order Matters

Arguments, call chains, and binary operations evaluate left-to-right. This is a
deliberate semantic choice because it:

- reduces surprising side-effect ordering,
- improves debugging predictability,
- stabilizes diagnostics and traceability.

When writing effectful code, rely on this guarantee intentionally.

## 12.3 `if` / `else`

Rules:

- condition must be `Bool`-compatible,
- exactly one branch executes,
- branch result types must be compatible where expression value is used.

Example:

```txt
pub classify(x: i64) -> Str {
  if x < 0 {
    "negative"
  } else if x == 0 {
    "zero"
  } else {
    "positive"
  }
}
```

## 12.4 `for ... in ...`

Syntax:

```txt
for item in iterable {
  ...
}
```

Semantics:

- iteration order follows iterable ordering contract,
- each iteration has its own loop-variable binding,
- termination occurs on iterator exhaustion.

Example:

```txt
pub sum(xs: List<i64>) -> i64 {
  total := 0i64
  for x in xs {
    total = total + x
  }
  total
}
```

## 12.5 `while`

`while` reevaluates condition before each iteration:

```txt
pub first_pow2_over(limit: i64) -> i64 {
  n := 1i64
  while n <= limit {
    n = n * 2
  }
  n
}
```

Keep side effects in conditions minimal for readability.

## 12.6 `repeat`

`repeat` is count-driven iteration:

```txt
repeat countExpr {
  ...
}
```

Key rules:

- count expression evaluated once,
- count must be integer and non-negative,
- body executes exactly count times.

Example:

```txt
pub ten_ticks() -> i64 {
  ticks := 0i64
  repeat 10 {
    ticks = ticks + 1
  }
  ticks
}
```

## 12.7 `break` and `continue`

- `break` exits nearest loop,
- `continue` moves to next iteration,
- using either outside loop context is a compile-time error.

Optional labels are part of the model for targeting explicit loops where needed.

## 12.8 `match`

`match` evaluates scrutinee once and checks arms top-to-bottom:

```txt
pub sign_label(x: i64) -> Str {
  match x {
    case 0 => "zero"
    case 1 => "one"
    default => "other"
  }
}
```

Use `default` unless checker can prove exhaustiveness.

## 12.9 `select` As Control Flow

`select` can be thought of as event-driven control flow:

```txt
select {
  case msg := inbox.recv() =>
    handle(msg)
  case after 250ms =>
    heartbeat()
  case closed inbox =>
    shutdown()
  case default =>
    idle()
}
```

Use `default` carefully. In tight loops, default-heavy selects can become busy
spins if not paired with throttling logic.

## 12.10 Tail Expressions vs Explicit Return

Use tail expressions for concise, obvious returns:

```txt
pub double(x: i64) -> i64 {
  x * 2
}
```

Use explicit return for branching and guard clauses:

```txt
pub safe_div(a: i64, b: i64) -> Result<i64, DivError> {
  if b == 0 {
    return err(DivError.zero_divisor())
  }
  ok(a / b)
}
```

## 12.11 Expression Complexity Heuristics

Readable code beats dense code in large systems. Practical heuristics:

- if a line has three or more nested operations, consider an intermediate binding,
- if a branch body performs multiple conceptual steps, expand to block form,
- avoid mixing control and transformation logic in one expression chain.

## 12.12 Control-Flow and Contracts

Contracts complement control flow:

- use `@require` to reject invalid branch space early,
- use `@ensure` to assert branch-merged invariants,
- use `@examples` to lock edge-case expectations.

This is especially helpful when logic includes early exits and nested loops.

## 12.13 Liveness and Termination Notes

The language semantics do not guarantee loop termination automatically. Avoid
implicit infinite loops unless you intentionally model services/consumers with
clear cancellation and shutdown behavior.

For long-running loops, always define:

- external stop condition,
- cancellation handling,
- timeout behavior where blocking I/O/concurrency is involved.

## 12.14 Common Control-Flow Mistakes

1. placing significant side effects in loop conditions,
2. forgetting to update loop state in `while`,
3. using `default` in `select` without backoff strategy,
4. writing non-exhaustive `match` without `default`,
5. mixing business logic and loop-control mechanics too tightly.

## 12.15 Clarification: Control Flow Is a Readability Contract

Control structures are not only execution tools; they are readability contracts
between authors and reviewers. Two implementations can be functionally
equivalent while having very different long-term maintainability. VibeLang's
structured forms (`for`, `while`, `repeat`, `match`, `select`) are designed so
teams can make intent visible in control shape itself.

If a reviewer cannot quickly explain why a branch or loop is safe, deterministic,
and terminating under expected conditions, the code is not yet "done" even if it
passes tests.

## 12.16 Chapter Checklist

You should now be able to:

- choose between `for`, `while`, and `repeat` intentionally,
- use `match` and `select` with deterministic semantics in mind,
- write clearer branching with tail expression and explicit returns,
- combine contracts with control logic to reduce regression risk.

---

Next: Chapter 13 dives into strings, containers, and data-structure semantics.
