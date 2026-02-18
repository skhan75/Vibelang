# Frontend Test Cases (v0.1)

## Purpose

This suite defines parser, binder, type checker, and diagnostics coverage for the frontend.

## Structure

- `parse_ok/*`: valid syntax parses successfully
- `parse_err/*`: invalid syntax emits expected parser diagnostics
- `type_ok/*`: type checking succeeds
- `type_err/*`: type checking fails with expected diagnostics
- `contract_ok/*`: valid contracts accepted
- `contract_err/*`: invalid contracts rejected

## Canonical Cases

## Parser Success

- Function declaration with contracts
- Method chaining expressions
- Select/go/channel forms
- Nested control flow with returns

## Parser Failure

- Missing closing brace
- Invalid `@examples` block structure
- Unknown annotation token shape

Expected behavior:

- Parser recovers and reports multiple errors in one pass.

## Type Checker Success

- Local type inference from `:=`
- Inferred list element type consistency
- Valid `Result<T,E>` propagation via `?`
- Valid postcondition type checking

## Type Checker Failure

- Unknown identifier in expression
- Assignment type mismatch
- Wrong argument count
- Invalid `.` usage outside `@ensure`
- `old(expr)` used in `@require`

## Contract Success

```txt
topK(xs: List<Int>, k: Int) -> List<Int> {
  @intent "k largest numbers, sorted desc"
  @examples {
    topK([3,1,2], 2) => [3,2]
  }
  @require k >= 0
  @ensure len(.) <= len(xs)
  @effect alloc
  xs.sort_desc().take(k)
}
```

## Contract Failure Cases

1. `@ensure` references unknown symbol
2. `@effect` value not in vocabulary
3. `@examples` contains non-call left side
4. Contract annotation appears after executable statement

## Diagnostics Golden Format

All failing cases assert:

- Diagnostic code
- Primary span
- Message text
- Optional fix-it suggestion text

Example:

```txt
E2104: invalid contract position: annotations must appear before executable statements
 --> app/math.vibe:12:3
```

## Test Matrix

| Area | Must Pass | Must Fail | Recovery Required |
| --- | --- | --- | --- |
| Lexer | yes | yes | yes |
| Parser | yes | yes | yes |
| Binder | yes | yes | no |
| Type Checker | yes | yes | no |
| Contracts | yes | yes | no |

## CI Policy

- Frontend tests run on every PR.
- Diagnostics golden tests are snapshot-based and require explicit approval for changes.
