# HIR Schema Notes and Verifier Checklist (Phase 1)

This document captures the initial typed HIR surface emitted by `vibe_types`.

## HIR Program

- `HirProgram`
  - `functions: Vec<HirFunction>`

## HIR Function

- `name: String`
- `is_public: bool`
- `params: Vec<HirParam>`
- `return_type: Option<TypeRef>`
- `inferred_return_type: Option<String>`
- `effects_declared: BTreeSet<String>`
- `effects_observed: BTreeSet<String>`

## HIR Parameter

- `name: String`
- `ty: Option<TypeRef>`

## Verifier Checklist

The Phase 1 verifier (`verify_hir`) must reject:

- empty function names
- duplicate function names in same HIR module

Additional checks planned post-Phase 1:

- incompatible inferred/decorated effect invariants
- unreachable synthetic check blocks
- cross-function symbol integrity
