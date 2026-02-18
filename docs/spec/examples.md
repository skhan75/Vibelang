# VibeLang Examples Spec (v0.1 Draft)

## Purpose

`@examples` provide executable examples colocated with implementation.
They serve as:

- Behavioral documentation
- Generated tests
- Regression guards during refactors

## Basic Form

```txt
@examples {
  call_expr => expected_expr
  call_expr2 => expected_expr2
}
```

## Rules

- Example entries must be deterministic.
- Expected value must be serializable for test output comparison.
- Expressions should avoid external I/O and time randomness.
- For floating-point values, v0.1 supports tolerance helper:
  - `approx(actual, expected, eps)`

## Lowering to Tests

Compiler creates synthetic test cases:

1. Resolve call expression and expected expression
2. Evaluate function under test
3. Compare with expected value
4. Emit pass/fail with source span

Generated tests are included in:

- `vibe test` default run
- CI by default unless disabled by profile

## Example Coverage Guidance

Recommended minimum per public function:

- One happy-path case
- One boundary case
- One empty/zero case (when applicable)

## Example: Collection Function

```txt
topK(xs, k) {
  @intent "k largest numbers, sorted desc"
  @examples {
    topK([3,1,2], 2) => [3,2]
    topK([], 5)      => []
    topK([1], 0)     => []
  }
  @ensure len(.) == min(k, len(xs))
  @ensure sorted_desc(.)
  @effect alloc

  xs.sort_desc().take(k)
}
```

## Example: Business Rule Function

```txt
fee(amount) {
  @intent "tiered percentage fee"
  @examples {
    fee(100)  => 2
    fee(1000) => 15
  }
  @require amount >= 0
  @ensure . >= 0

  if amount < 500 {
    amount * 0.02
  } else {
    amount * 0.015
  }
}
```

## Failure Reporting

Example test failures should include:

- Function name
- Input values
- Expected vs actual
- Source span to example entry

Sample:

```txt
Example failed: topK([3,1,2], 2) => [3,2]
actual: [3,1]
at app/math.vibe:12:5
```

## Anti-Patterns

- Too many redundant examples with no new signal
- Non-deterministic examples (time/network/random)
- Encoding giant integration tests inside function annotations
