# VibeLang ABI and FFI Spec (v1.0 Target)

Status: normative target.

## Scope

- Defines external calling conventions and data layout assumptions.
- Defines safety constraints for foreign-function boundaries.

## Calling Convention

- External boundary default uses platform C ABI unless explicitly overridden.
- ABI contract must specify:
  - parameter passing mode
  - return value convention
  - stack alignment rules

## Data Layout Rules

- Primitive type sizes follow `numeric_model.md`.
- Struct/type field layout follows deterministic declaration order unless
  explicitly packed/reordered by attribute.
- Alignment and padding must be deterministic for target ABI.

## Endianness

- Endianness follows target platform ABI.
- Cross-endian interop requires explicit conversion helpers.

## String and Buffer Interop

- `Str` ABI representation must be explicit at FFI boundary (pointer+len or
  equivalent contract).
- Ownership transfer at FFI boundary must be explicit (borrow/copy/move).

## Error Interop

- Foreign errors must map into VibeLang error channels deterministically.
- Panic/trap crossing foreign boundary is forbidden unless explicit boundary
  policy defines behavior.

## Threading and Reentrancy

- FFI functions called from runtime threads must satisfy thread-safety contract.
- Callbacks into VibeLang must respect runtime scheduler/GC safepoint rules.

## Unsafe Boundary Policy

- FFI is unsafe by default unless wrapped in safe abstraction with validated
  preconditions.
- Safe wrappers SHOULD perform argument/lifetime checks.

## Determinism Requirements

- ABI codegen for same source/target/profile must be reproducible.
- FFI diagnostics must include boundary symbol and deterministic error code.

## Target Support Baseline

Initial baseline target set aligns with current runtime/codegen support policy
and release docs. Any ABI guarantee claim must specify supported target triples.

## Deferred Notes

- Stable binary module/plugin ABI is deferred unless explicitly accepted.
