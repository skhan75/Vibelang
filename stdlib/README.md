# VibeLang stdlib (Phase 12)

Phase 12 expands the stdlib from a minimal I/O core into a practical day-to-day surface while
keeping deterministic behavior and local-first validation.

## Module index

- `io` (stable): `print`, `println`
- `core` (stable/preview): deterministic utility APIs for contract/example validation
- `time` (preview): clock + duration helpers
- `path` (stable): path composition/introspection helpers
- `fs` (preview): filesystem read/write/exists/directory helpers
- `net` (preview): TCP + DNS primitives (`listen/connect/read/write/close/resolve`)
- `convert` (preview): parsing + formatting conversions (`to_int/to_float/to_str`)
- `text` (preview): text utilities (`trim/contains/replace/case/split_part/index_of`)
- `str_builder` (preview): growable string buffer (`new`/`append`/`append_char`/`finish`)
- `regex` (preview): pattern helpers (`count`/`replace_all`)
- `encoding` (preview): hex/base64/url encode/decode helpers
- `json` (preview): `Json` parse/stringify (+ pretty), `json.builder` for dynamic JSON, typed codecs, utilities (`is_valid`, `parse_i64`/`stringify_i64`, `minify`)
- `http` (preview): sync request client + protocol helpers
- `log` (preview): structured-ish log primitives (`info/warn/error`)
- `env` (preview): environment variable accessors
- `cli` (preview): process argument helpers
- `bench` (benchmark-only): gated benchmark parity APIs (requires `bench-runtime`)

Detailed module references:

- `stdlib/io/README.md`
- `stdlib/core/deterministic_apis.md`
- `stdlib/time/README.md`
- `stdlib/path/README.md`
- `stdlib/fs/README.md`
- `stdlib/net/README.md`
- `stdlib/convert/README.md`
- `stdlib/text/README.md`
- `stdlib/str_builder/README.md`
- `stdlib/regex/README.md`
- `stdlib/encoding/README.md`
- `stdlib/json/README.md`
- `stdlib/http/README.md`
- `stdlib/log/README.md`
- `stdlib/env/README.md`
- `stdlib/cli/README.md`
- `stdlib/bench/README.md`

## Compiler/runtime contract

- Typechecker recognizes stdlib namespace calls (`time.*`, `path.*`, `fs.*`, `net.*`, `convert.*`, `text.*`, `str_builder.*`, `regex.*`, `encoding.*`, `json.*`, `http.*`, `log.*`, `env.*`, `cli.*`)
  and enforces argument/return contracts.
- With `bench-runtime`, typechecker/codegen also recognize `bench.*` and lower those calls to
  `vibe_bench_*` symbols in `runtime/native/vibe_runtime_bench.c`.
- Codegen lowers the default stdlib calls to runtime `vibe_*` symbols in `runtime/native/vibe_runtime.c`.
- Runtime implementations are deterministic for equal inputs except explicitly nondeterministic
  APIs (`time.now_ms`, `time.monotonic_now_ms`, `env.*`, `cli.*`).

## Versioning and compatibility

- Stability tiers and compatibility rules: `stdlib/stability_policy.md`
- Reference index and versioning guarantees: `docs/stdlib/reference_index.md`,
  `docs/stdlib/versioning_guarantees.md`
