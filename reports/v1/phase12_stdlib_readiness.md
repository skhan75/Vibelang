# Phase 12.1 Stdlib Readiness (Local-First)

Date: 2026-02-17

## Status

- Result: `LOCAL-PASS`
- Scope: filesystem/path, time/duration, json serialization helpers, http protocol essentials

## Implemented surface

- Typechecker + diagnostics:
  - stdlib namespace call contracts for `time.*`, `path.*`, `fs.*`, `json.*`, `http.*`
- Native codegen/runtime:
  - runtime symbols added for all above module essentials
  - codegen lowering dispatch for namespace calls
- Docs + policy:
  - `stdlib/stability_policy.md`
  - `stdlib/{time,path,fs,json,http}/README.md`
  - `docs/stdlib/reference_index.md`
  - `docs/stdlib/versioning_guarantees.md`

## Local validation evidence

- `cargo test -p vibe_cli --test phase12_stdlib`
  - pass: `4 passed; 0 failed`
  - covers:
    - end-to-end module surface execution
    - deterministic repeat-run behavior
    - error-model stability
    - invalid-call diagnostics
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/time --json`
  - pass: json summary reports zero failures

## Fixture corpus

- `compiler/tests/fixtures/stdlib/time/basic.yb`
- `compiler/tests/fixtures/stdlib/path/basic.yb`
- `compiler/tests/fixtures/stdlib/fs/basic.yb`
- `compiler/tests/fixtures/stdlib/json/basic.yb`
- `compiler/tests/fixtures/stdlib/http/basic.yb`
