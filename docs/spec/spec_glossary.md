# VibeLang Spec Glossary

This glossary defines canonical terms used across the specification suite.

## Language Terms

- Binding: A named value introduced in scope.
- Immutable binding: Binding that cannot be reassigned after initialization.
- Mutable binding: Binding that can be reassigned after initialization.
- Tail expression: Final expression in a block used as implicit return value.
- Contract: Executable metadata (`@require`, `@ensure`, `@examples`, `@effect`)
  attached to function behavior.
- Intent: Human-readable behavior objective attached via `@intent`.
- Effect: Observable side-effect category declared via `@effect`.

## Type System Terms

- Primitive type: Built-in scalar type (for example `i32`, `f64`, `Bool`).
- Aggregate type: Composite type (`List<T>`, `Map<K,V>`, structs).
- Monomorphization: Compile-time specialization of generic code for concrete
  type arguments.
- Assignability: Rule deciding whether a value of one type may be bound or
  assigned to another type.
- Coercion: Implicit type conversion allowed by language rules.
- Cast: Explicit type conversion requested by user syntax.

## Numeric Terms

- Width: Number of bits in numeric storage (`i32` is 32-bit).
- Overflow: Arithmetic result outside representable range of type.
- Wrap-around: Overflow behavior where result is reduced modulo 2^N.
- Saturating arithmetic: Overflow clamps to min or max representable value.
- IEEE-754: Float behavior standard used by `f32` and `f64`.
- NaN: Not-a-Number float value with special comparison behavior.

## Text And Container Terms

- Code point: Unicode scalar value.
- Grapheme cluster: User-perceived character that may span multiple code points.
- UTF-8: Canonical byte encoding for `Str`.
- Stable iteration order: Container iteration order guaranteed to remain
  deterministic for same inputs.

## Concurrency And Runtime Terms

- Task: Lightweight scheduled unit created by `go` or async runtime.
- Thread: OS-managed execution thread.
- M:N scheduler: Runtime mapping many language tasks onto fewer OS threads.
- Happens-before: Visibility ordering relation between memory actions.
- Synchronization point: Operation that establishes happens-before (for example
  channel send/recv pair).
- Sendability: Property that value may cross task/thread boundary safely.
- Cancellation token: Signal used to cooperatively stop work.

## Build And Conformance Terms

- Normative: Defines required language behavior.
- Explanatory: Guidance, examples, or rationale; not the final authority.
- Deferred: Accepted scope item with explicit future target; not implemented in
  current release profile.
- Spec integrity gate: CI gate that validates spec consistency and traceability.
