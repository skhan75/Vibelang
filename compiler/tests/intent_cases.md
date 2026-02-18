# Intent/Contract Engine Test Cases (v0.1)

## Contract Parsing

- Parse single `@intent`
- Parse multi-case `@examples`
- Parse multiple `@ensure` lines
- Reject unknown annotation names

## Semantic Validation

- `@require` cannot reference `.` result placeholder
- `@ensure` may reference `.` and `old(...)`
- `old(...)` only valid in `@ensure`
- `@effect` must belong to known vocabulary

## Generated Tests

Given:

```txt
sum(xs) {
  @examples {
    sum([1,2,3]) => 6
    sum([]) => 0
  }
  xs.reduce(0, add)
}
```

Expected:

- Two generated test cases in synthetic module
- Stable test names (hash includes file + function + case index)
- Failure output includes source span for case

## Determinism Enforcement

Fail contract when:

- `@ensure` calls random/time API
- `@examples` expected value uses nondeterministic operation

## Effect Matching

Pass:

- Declared `@effect alloc` and allocation observed

Fail:

- Observed `io` but missing `@effect io`
- Declared `@effect io` but no I/O observed (warning)

## Golden Diagnostics

Validate diagnostic code, message, and span for:

- malformed `@examples`
- invalid `old(...)`
- unknown effect
- postcondition expression type mismatch
