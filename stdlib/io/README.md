# `stdlib/io` (Phase 2)

This directory documents the initial IO contract for Phase 2 native execution.

## Public surface

- `print(Str) -> Void`
- `println(Str) -> Void`

## Current implementation path

1. parser + type checker accept builtin calls to `print` and `println`
2. HIR -> MIR preserves these calls by name
3. Cranelift codegen lowers to imported runtime function `vibe_println(const char*)`
4. runtime C shim prints to stdout with a trailing newline

## Notes

- `print` and `println` are currently equivalent in Phase 2
- non-string arguments are not supported yet
- richer IO API design is deferred to later phases
