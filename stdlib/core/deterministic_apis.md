# Deterministic Utility APIs (Phase 2B)

These APIs define the deterministic baseline used by contract/example execution in Phase 2B.

## Purpose

- power `@examples` execution in `vibe test`
- keep behavior machine-independent and reproducible
- avoid hidden I/O, time, randomness, and network dependencies

## API Set

- `len(value) -> Int`
  - supports `List` and `Str`
- `min(a: Int, b: Int) -> Int`
- `max(a: Int, b: Int) -> Int`
- `sorted_desc(xs: List<Int>) -> Bool`
- `sort_desc(xs: List<Int>) -> List<Int>`
- `take(xs: List<T>, n: Int) -> List<T>`

## Determinism Rules

- no wall-clock access
- no randomness
- no process/environment-dependent values
- stable ordering and pure outputs for same inputs
- `cpu_count()` in example runner is fixed to `1` for deterministic checks

## Phase 2B Scope Notes

- these utilities are currently implemented in the CLI deterministic example runner path
- this is a bootstrap implementation for checks/examples, not final stdlib ABI
- richer generic/container APIs are deferred to later phases
