# VibeLang stdlib (Phase 2)

Phase 2 intentionally keeps the standard library surface minimal so the native backend can stabilize around a small, deterministic core.

## Included in Phase 2

- `io.print(value: Str) -> Void`
- `io.println(value: Str) -> Void`

In this phase, these are compiler-recognized builtins:

- type checking treats `print` and `println` as known symbols that return `Void`
- codegen lowers both to the runtime symbol `vibe_println`
- runtime provides `vibe_println` via `runtime/native/vibe_runtime.c`

## Deterministic utilities for checks/examples

Phase 2B adds deterministic utility APIs used by contract/example execution (`vibe test`):

- `len`
- `min`
- `max`
- `sorted_desc`
- `sort_desc`
- `take`

Reference: `stdlib/core/deterministic_apis.md`

## Stability policy

Reference: `stdlib/stability_policy.md`

## Boundaries and non-goals

- no formatting APIs (`printf`, interpolation helpers, width/precision controls)
- no file I/O abstraction yet
- no buffered writer API yet
- no user-extensible IO traits/protocols yet
- no allocator/runtime GC hooks exposed through stdlib in Phase 2

## Rationale

The goal is to make hello-world and basic native execution work first, while keeping the runtime contract tiny and auditable.
