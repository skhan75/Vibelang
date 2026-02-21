# V1 Dynamic Containers Conformance Report

Date: 2026-02-20

## Objective

Record local release-candidate evidence for the `7.3.f.1` implementation freeze:
native `Str`/`List`/`Map` construction and container/member operations required for
v1 GA blockers.

## Supported Surface (7.3.f.1 Freeze)

Reference source of truth: `docs/spec/containers.md` section
`7.3.f.1 Implementation Support Freeze (v1 GA Blocker Scope)`.

- `List<Int>`: literal construction, `append`, `get`, `set`, `len`.
- `Map<Int, Int>`: literal construction, `get`, `set`, `contains`, `remove`, `len`.
- `Map<Str, Int>`: same API as `Map<Int, Int>` with string keys.
- `Str`: literal construction and concatenation (`Str + Str`).

## Local Evidence Commands

All commands were executed from workspace root:

```bash
cargo test -p vibe_cli --test frontend_fixtures parse_err_golden
cargo test -p vibe_cli --test frontend_fixtures type_ok_fixtures
cargo test -p vibe_cli --test frontend_fixtures type_err_golden
cargo test -p vibe_cli --test frontend_fixtures ownership_err_golden
cargo test -p vibe_cli --test frontend_fixtures snapshots_container_ops_mir_is_deterministic

cargo test -p vibe_cli --test phase7_v1_tightening phase7_algorithmic_recursion_samples_run_expected_outputs
cargo test -p vibe_cli --test phase7_v1_tightening phase7_memory_heap_pressure_smoke_is_bounded
cargo test -p vibe_cli --test phase7_v1_tightening phase7_ownership_sendability_smokes_cover_positive_and_negative_paths
```

## Result Summary

- Parser diagnostics for map/list forms and invalid literals: PASS.
- Type-check coverage for container method signatures and key/value compatibility: PASS.
- Ownership/sendability checks for container values crossing `go` boundaries: PASS.
- MIR determinism for representative container sample (`List` + `Map`): PASS.
- Runtime algorithmic conformance (Catalan/generate-parentheses count fixture): PASS.
- Runtime bounded memory smoke for container pressure loop: PASS.

## CI Gate Integration

- Blocking workflow gate: `.github/workflows/v1-release-gates.yml` job
  `dynamic_containers_gate`.
- Summary dependency wired via workflow `summary.needs`.
- Report existence is enforced by workflow job `reports_gate`.

## Remaining Deferred Items

Deferred items explicitly outside `7.3.f.1` freeze remain tracked in
`docs/spec/containers.md`:

- Generic native lowering beyond the supported concrete combinations.
- String indexing/slicing APIs.
- Container iterators and native `for in` lowering over dynamic containers.
- Broader container equality/hash APIs not required by v1 smoke matrix.
