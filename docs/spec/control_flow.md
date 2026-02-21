# VibeLang Control Flow Spec (v1.0 Target)

Status: normative target.

## Supported Control Forms

- `if` / `else`
- `for name in iterable`
- `while condition`
- `repeat count`
- `match expr { ... }`
- `break` / `continue` (optional labels)
- `return`

## Block and Scope Rules

- Every control construct body is a brace-delimited block.
- Each block introduces a new lexical scope.
- Shadowing inside nested scopes is legal.

## `if` Semantics

- Condition expression must evaluate to `Bool` (or type-check compatible).
- Exactly one branch executes.
- Branch expression types must be compatible where expression result is used.
- `else if` is syntactic sugar for nested `if`.

## `for` Semantics

Syntax:

```txt
for item in iterable {
  ...
}
```

Rules:

- Iteration order must follow iterable's defined ordering semantics.
- Loop variable binding is per-iteration scoped binding.
- Loop terminates when iterator reports exhaustion.

## `while` Semantics

Syntax:

```txt
while condition {
  ...
}
```

Rules:

- Condition is re-evaluated before each iteration.
- Loop exits when condition evaluates to `false`.
- Side effects in condition are permitted but discouraged for clarity.

## `repeat` Semantics

Syntax:

```txt
repeat countExpr {
  ...
}
```

Rules:

- `countExpr` is evaluated once before loop begins.
- Count must be integer and non-negative.
- Loop body executes exactly `countExpr` times for valid count.
- Negative count is compile-time error if provable, otherwise deterministic
  runtime trap.

## `break` and `continue`

- `break` exits nearest enclosing loop by default.
- `continue` jumps to next iteration of nearest enclosing loop by default.
- Optional label form (`break label`, `continue label`) targets named loop.
- Using `break`/`continue` outside loop context is compile-time error.

## `match` Semantics

Syntax:

```txt
match expr {
  case pattern1 => action1
  case pattern2 => action2
  default => fallback
}
```

Rules:

- Scrutinee evaluated once.
- Arms evaluated top-to-bottom; first matching arm executes.
- `default` arm is required unless checker can prove exhaustiveness.
- Arm action type compatibility follows expression-context rules.

## `return` and Tail Expressions

- `return expr` exits current function immediately.
- If no explicit terminal `return`, final block expression is implicit return.
- Functions with declared return type must satisfy assignability of all return
  paths.

## Select-Loop Interaction

- `break` inside `select` case affects nearest enclosing loop, not `select`
  itself, unless language introduces `break select` label form.
- `continue` in `select` case follows same nearest-loop targeting.

## Termination and Liveness Notes

- Language does not guarantee loop termination; semantics describe behavior, not
  static totality proof.
- Runtime may provide diagnostics for likely non-terminating loops in tooling
  mode.

## Determinism Requirements

- Control-flow evaluation order and branch selection must be deterministic for
  deterministic inputs.
- Diagnostic ordering for control-flow errors must be stable.

## Deferred Notes

- Pattern guards in `match` are deferred unless added in grammar decision log.
