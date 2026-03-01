# VibeLang stdlib (Phase 12)

Phase 12 expands the stdlib from a minimal I/O core into a practical day-to-day surface while
keeping deterministic behavior and local-first validation.

## Module index

- `io` (stable): `print`, `println`
- `core` (stable/preview): deterministic utility APIs for contract/example validation
- `time` (preview): clock + duration helpers
- `path` (stable): path composition/introspection helpers
- `fs` (preview): filesystem read/write/exists/directory helpers
- `json` (preview): parse/validate/minify/stringify primitives
- `http` (preview): protocol helper primitives (status text, default ports, request lines)
- `bench` (benchmark-only): gated benchmark parity APIs (requires `bench-runtime`)

Detailed module references:

- `stdlib/io/README.md`
- `stdlib/core/deterministic_apis.md`
- `stdlib/time/README.md`
- `stdlib/path/README.md`
- `stdlib/fs/README.md`
- `stdlib/json/README.md`
- `stdlib/http/README.md`
- `stdlib/bench/README.md`

## Compiler/runtime contract

- Typechecker recognizes stdlib namespace calls (`time.*`, `path.*`, `fs.*`, `json.*`, `http.*`)
  and enforces argument/return contracts.
- With `bench-runtime`, typechecker/codegen also recognize `bench.*` and lower those calls to
  `vibe_bench_*` symbols in `runtime/native/vibe_runtime_bench.c`.
- Codegen lowers the default stdlib calls to runtime `vibe_*` symbols in `runtime/native/vibe_runtime.c`.
- Runtime implementations are deterministic for equal inputs except explicitly nondeterministic
  APIs (`time.now_ms`).

## Versioning and compatibility

- Stability tiers and compatibility rules: `stdlib/stability_policy.md`
- Reference index and versioning guarantees: `docs/stdlib/reference_index.md`,
  `docs/stdlib/versioning_guarantees.md`
