# Phase 12.1 Stdlib Readiness (Local-First)

Date: 2026-03-01

## Status

- Result: `LOCAL-PASS`
- Scope: time/path/fs/net/convert/text/encoding/json/http/log/env/cli stdlib surfaces

## Known preview constraints

- JSON/convert/encoding decode-style error paths currently use sentinel returns instead of
  Result-returning contracts.
- HTTP surface is sync-first (`request/request_status/get/post`) and does not yet expose a
  structured request/response type in language space.

## Implemented surface

- Typechecker + diagnostics:
  - stdlib namespace call contracts for:
    - `time.*` (including `monotonic_now_ms`)
    - `path.*`, `fs.*`, `net.*`
    - `convert.*`, `text.*`, `encoding.*`
    - `json.*` (including typed codecs: `encode_<Type>`, `decode_<Type>`)
    - `http.*` (sync client surface)
    - `log.*`, `env.*`, `cli.*`
- Native codegen/runtime:
  - runtime symbols added for all above module surfaces
  - codegen lowering dispatch for namespace calls
- Docs + policy:
  - `stdlib/stability_policy.md`
  - `stdlib/{time,path,fs,net,convert,text,encoding,json,http,log,env,cli}/README.md`
  - `docs/stdlib/reference_index.md`
  - `docs/stdlib/versioning_guarantees.md`

## Local validation evidence

- `cargo test -p vibe_cli --test phase12_stdlib`
  - pass: `4 passed; 0 failed`
  - covers:
    - end-to-end module surface execution (including local deterministic net/http lanes)
    - deterministic repeat-run behavior
    - error-model stability
    - invalid-call diagnostics
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/time --json`
  - pass: json summary reports zero failures
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/path --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/fs --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/json --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/http --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/convert --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/text --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/encoding --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/env_cli --json`
- `cargo run -q -p vibe_cli -- test compiler/tests/fixtures/stdlib/log --json`

## Fixture corpus

- `compiler/tests/fixtures/stdlib/time/basic.yb`
- `compiler/tests/fixtures/stdlib/path/basic.yb`
- `compiler/tests/fixtures/stdlib/fs/basic.yb`
- `compiler/tests/fixtures/stdlib/json/basic.yb`
- `compiler/tests/fixtures/stdlib/http/basic.yb`
- `compiler/tests/fixtures/stdlib/convert/basic.yb`
- `compiler/tests/fixtures/stdlib/text/basic.yb`
- `compiler/tests/fixtures/stdlib/encoding/basic.yb`
- `compiler/tests/fixtures/stdlib/env_cli/basic.yb`
- `compiler/tests/fixtures/stdlib/log/basic.yb`
