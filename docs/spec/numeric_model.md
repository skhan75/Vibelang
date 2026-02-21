# VibeLang Numeric Model (v1.0 Target)

Status: normative target.

## Numeric Type Families

### Signed Integers

- `i8`  (8-bit)
- `i16` (16-bit)
- `i32` (32-bit)
- `i64` (64-bit)
- `isize` (pointer-width signed integer)

### Unsigned Integers

- `u8`  (8-bit)
- `u16` (16-bit)
- `u32` (32-bit)
- `u64` (64-bit)
- `usize` (pointer-width unsigned integer)

### Floating Point

- `f32` (IEEE-754 binary32)
- `f64` (IEEE-754 binary64)

## Representation and Layout

- Fixed-width integer types MUST use two's-complement representation.
- `isize`/`usize` width MUST match target pointer width.
- Endianness follows target platform ABI.
- Type alignment and layout follow `docs/spec/abi_and_ffi.md`.

## Literal Typing Rules

- Integer literal default type: `i64` unless context requires another integer
  type.
- Float literal default type: `f64` unless context requires `f32`.
- Suffix forms are authoritative:
  - integer suffixes: `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`,
    `u64`, `usize`
  - float suffixes: `f32`, `f64`
- Literal out-of-range for target type is compile-time error.

Examples:

```txt
count: u32 := 10u32
small: i8 := 12i8
ratio: f32 := 0.5f32
pi := 3.1415926535        # inferred f64 by default
```

## Integer Arithmetic Semantics

### Checked Contexts (Default in Dev/Test)

- `+`, `-`, `*`, `/`, `%` on integer types MUST trap with deterministic runtime
  error on overflow/underflow/divide-by-zero unless operation is compile-time
  provably safe.

### Release Profile Policy

- Default release policy MUST remain deterministic and explicit in build profile
  settings.
- Supported release policies:
  - `checked` (trap on overflow)
  - `wrapping` (modulo 2^N semantics)
  - `saturating` (clamp to min/max)
- Toolchain default policy for GA MUST be documented in release notes and
  profiles.

## Float Arithmetic Semantics

- Follows IEEE-754 for `f32` and `f64`.
- Supports `NaN`, `+Inf`, `-Inf`, signed zero.
- Division by zero for floats yields IEEE result (not integer-style trap).
- NaN is not equal to any value, including itself.
- Total ordering helpers (if provided) MUST define NaN ordering policy.

## Rounding and Conversion

- Float-to-int conversion requires explicit cast/conversion and MUST define
  rounding mode.
- Default conversion mode for `as`-style cast (if enabled) is truncation toward
  zero.
- Out-of-range float-to-int conversion MUST be deterministic (trap, saturate, or
  error return according to conversion API contract).

## Comparison Semantics

- Integer comparisons are exact.
- Float comparisons follow IEEE partial ordering.
- Exact equality on floats is allowed but discouraged for tolerance-sensitive
  domains; use approximate comparison helpers when needed.

## Numeric Coercion Rules

- Implicit widening allowed:
  - narrower int -> wider int of same signedness
  - int -> float only where explicitly allowed by type checker rule
- Implicit narrowing is forbidden.
- Signed/unsigned mixing requires explicit conversion unless exact safe coercion
  rule is defined.

## Compile-Time Constant Evaluation

- Constant-folding MUST match runtime semantics for the same profile policy.
- Overflow behavior during constant evaluation MUST use same policy as runtime
  mode selected for the build profile.

## Diagnostics

Numeric diagnostics SHOULD include:

- source operation and operand types
- overflow/divide-by-zero context
- suggested fix (change type, explicit conversion, policy-aware helper)

## Deferred Notes

- Decimal fixed-point and big-integer types are deferred beyond v1 target unless
  explicitly accepted by decision log.
