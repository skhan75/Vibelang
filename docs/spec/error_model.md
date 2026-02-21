# VibeLang Error Model (v1.0 Target)

Status: normative target.

## Error Channels

VibeLang supports explicit error channels:

- `Result<T,E>` for recoverable failures.
- Contract violations for failed `@require`/`@ensure`.
- Runtime trap/panic for unrecoverable internal failures.

## `Result<T,E>` Semantics

- Functions returning `Result<T,E>` must return `ok(value)` or `err(error)`.
- `?` operator is valid for `Result<T,E>`:
  - on `ok(v)`: unwraps to `v`
  - on `err(e)`: returns early with compatible error channel

Nested error-channel conversion requires explicit mapping APIs.

## Contract Failure Semantics

- `@require` failure: entry precondition violation.
- `@ensure` failure: exit postcondition violation.
- In dev/test profiles, contract failure is hard failure with diagnostic context.
- In release profile, behavior is profile policy (`strict`/`balanced`/`off`
  variants) but must remain deterministic.

## Panic/Trap Semantics

- Panic/trap represents unrecoverable execution failure.
- Default behavior for panic in task/thread contexts must be explicit:
  - fail current task
  - propagate to parent scope (structured mode)
  - terminate process when unhandled at top boundary

## Async and Concurrency Propagation

- Task failure propagation:
  - joined task returns failure to join caller
  - unjoined detached task follows runtime unhandled-failure policy
- Async await failure propagates through await expression according to awaited
  type (`Result` or panic channel).

## Error Value Requirements

Error types SHOULD provide:

- stable code/classification
- human-readable message
- contextual metadata where safe

## Diagnostics and Reporting

Error diagnostics SHOULD include:

- error channel (`Result`/contract/panic)
- source span and operation
- boundary context (task/thread/async)
- deterministic code and message template

## Interop Boundaries

- FFI-originated errors must be translated into VibeLang error channels
  deterministically.
- Lossy or opaque foreign errors must preserve at least stable category.

## Deferred Notes

- Exception-style implicit unwinding model is out-of-scope for v1 safe surface.
