# VibeLang Features and Optimizations Checklist

Last updated: 2026-03-25

## Purpose

This is the canonical implementation checklist for feature gaps, limitations, and optimization items needed for VibeLang to reach mature production parity.

## Semantic-First Example Policy

- Examples must represent intended VibeLang semantics and ergonomics.
- Do not rewrite examples into workaround form only to pass current runtime limitations.
- If an example fails due to language/runtime/tooling limitations, keep the semantic example and track the gap here with an actionable fix item.
- Every failing example should map to at least one checklist item ID.

## Current Snapshot

- Example corpus: `81` programs under `examples/` (added 56–58 for text.index_of, unfurld smoke, Map<Str,Str>+json.from_map)
- Static status: all examples pass `vibe check`
- Runtime status (source-built CLI sweep): `73` pass / `5` fail
- Non-entry helper module files now fail with explicit entrypoint diagnostics (expected):
  - `examples/08_modules_packages/project_math/demo/math.yb`
  - `examples/08_modules_packages/project_pipeline/app/parser.yb`
  - `examples/08_modules_packages/project_pipeline/app/formatter.yb`
- Intentional failure demos (keep failing): `examples/10_contracts_intent/68_runtime_require_failure_demo.yb`, `examples/10_contracts_intent/69_runtime_ensure_failure_demo.yb`
- Release gate status: GA gates closed in `reports/v1/readiness_dashboard.md`
- Benchmark strict-publication status: tracked in `docs/checklists/benchmarks.md`

---

## A) Runtime and Codegen Parity Gaps (Examples Execution)

### A-01 (P0) `Str.len()` runtime parity
- [x] Implement stable lowering/runtime dispatch for string length.
- [x] Fix `.len()` on struct `Str` fields (was dispatching to `container_len` instead of `str_len_bytes`).
- **Symptoms**: `panic: container len called on unsupported container`
- **Root cause (struct fields)**: `is_known_string_expr_with_owner` did not recognize `MirExpr::Member` with `Str`-typed fields. Fixed by adding `is_known_string_expr_full` that checks field types against `type_defs`.
- **Impacted examples**:
  - `examples/02_strings_numbers/11_string_len_compare.yb`
  - `examples/02_strings_numbers/12_string_build_loop.yb`
  - `examples/05_graphs_recursion_patterns/36_palindrome_check.yb`
  - `examples/08_modules_packages/project_pipeline/app/main.yb` (via `parser.yb`)
- **Likely subsystem**: codegen + runtime dispatch
- **Evidence**: `crates/vibe_codegen/src/lib.rs` (`is_known_string_expr_full`), `examples/07_stdlib_io_json_regex_http/58_map_str_str_json_from_map.yb`
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - `struct_field.len()` returns correct byte length
  - Add integration tests for `s.len()` in loops and helper functions

### A-01a (P0) `Str` equality on struct fields and dynamic strings
- [x] Fix `==` / `!=` on `Str` fields to use value comparison (`vibe_str_eq`) instead of pointer comparison (`icmp`).
- [x] Fix string `+` concatenation with struct `Str` fields to use `vibe_str_concat`.
- **Symptoms**: `struct.field == "literal"` always returned false; `"prefix" + struct.field` produced garbage.
- **Root cause**: Same as A-01 — `is_known_string_expr_with_owner` did not recognize `MirExpr::Member`.
  Additionally, `Eq`/`Ne` required BOTH sides to be known strings; relaxed to EITHER side.
- **Evidence**: `crates/vibe_codegen/src/lib.rs` (`is_known_string_expr_full`), `examples/07_stdlib_io_json_regex_http/58_map_str_str_json_from_map.yb`
- **Acceptance**:
  - `struct_field == "literal"` performs content comparison
  - `"prefix" + struct_field` produces correct concatenation

### A-02 (P0) List method dispatch parity (`.get` / `.set`)
- [x] Fix list receiver dispatch so list methods never route to map/string-key paths.
- **Symptoms**: `panic: container get(Str)...`, `panic: container set(Str, Int)...`
- **Impacted examples**:
  - `examples/03_data_structures/16_list_append_get_set.yb`
  - `examples/03_data_structures/19_queue_with_list.yb`
  - `examples/04_algorithms/23_linear_search.yb`
  - `examples/04_algorithms/24_binary_search.yb`
  - `examples/04_algorithms/25_bubble_sort.yb`
  - `examples/04_algorithms/26_selection_sort.yb`
  - `examples/04_algorithms/27_prefix_sum.yb`
  - `examples/05_graphs_recursion_patterns/32_graph_bfs_small.yb`
  - `examples/05_graphs_recursion_patterns/34_tree_sum_recursive.yb`
  - `examples/09_agentic_patterns/47_agentic_guardrail_pipeline.yb`
- **Likely subsystem**: runtime method dispatch + codegen receiver typing
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - Add list method conformance tests (index patterns, loops, recursion)

### A-03 (P0) `Map<Int, Int>` method parity
- [x] Implement correct key-type dispatch for integer-key maps (`get/set/contains/remove`).
- **Symptoms**: int-key map calls route through str-key runtime and panic.
- **Impacted examples**:
  - `examples/03_data_structures/21_map_int_int_basics.yb`
  - `examples/05_graphs_recursion_patterns/33_graph_dfs_small.yb`
- **Acceptance**:
  - Both examples pass `vibe run`
  - Add explicit `Map<Int, Int>` runtime conformance tests for all methods

### A-04 (P0) Missing builtin lowering (`max`, global `len`)
- [x] Implement/restore codegen lowering for builtins referenced in language surface.
- **Symptoms**: `E3403: unknown call target`
- **Impacted examples**:
  - `examples/02_strings_numbers/13_int_arithmetic_min_max.yb` (`max`)
  - `examples/05_graphs_recursion_patterns/35_tree_depth_recursive.yb` (`max`)
  - `examples/10_contracts_intent/66_list_transform_contracts.yb` (`len`)
  - `examples/10_contracts_intent/67_public_api_style_contracts.yb` (`len`)
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - Add builtins smoke tests for `min/max/len` under `vibe run` and `vibe test`

### A-05 (P0) `.sort_desc()` native backend support
- [x] Implement list sort lowering in native backend for supported list types.
- **Symptoms**: `E3404: member call .sort_desc() is not supported in v0.1 native backend`
- **Impacted examples**:
  - `examples/03_data_structures/17_list_sort_take.yb`
  - `examples/10_contracts_intent/66_list_transform_contracts.yb`
  - `examples/10_contracts_intent/67_public_api_style_contracts.yb`
- **Acceptance**:
  - All impacted examples pass `vibe run`
  - Deterministic ordering tests added for sorted output

### A-06 (P0) Float codegen/runtime stability
- [x] Fix float value typing/lowering and verifier failures.
- **Symptoms**: verifier errors / backend panic on float examples.
- **Impacted examples**:
  - `examples/02_strings_numbers/14_float_basics.yb`
  - `examples/02_strings_numbers/15_float_comparison.yb`
- **Acceptance**:
  - Both examples pass `vibe run`
  - Add float arithmetic/comparison integration tests

### A-07 (P0) Contract example-runner parity
- [x] Align `vibe test` example evaluator with executable language surface.
- **Symptoms**: contract/example preflight rejects methods used in regular code.
- **Impacted examples**:
  - `examples/10_contracts_intent/60_effect_alloc_mut_state.yb`
- **Acceptance**:
  - Contract examples using list/map methods execute under `vibe test`
  - If a subset is intentionally restricted, enforce with explicit diagnostics and docs

### A-08 (P1) Module helper-file run ergonomics
- [x] Improve CLI error for non-entry module execution.
- **Symptoms**: link error `undefined reference to main` when running helper module files.
- **Impacted examples**:
  - `examples/08_modules_packages/project_math/demo/math.yb`
  - `examples/08_modules_packages/project_pipeline/app/parser.yb`
  - `examples/08_modules_packages/project_pipeline/app/formatter.yb`
- **Acceptance**:
  - User-friendly diagnostic points to module entry file
  - Docs include explicit entrypoint run guidance

---

## B) Benchmark Publication and Performance Blockers

Benchmark publication and benchmarking execution checklists are maintained in one place:

- `docs/checklists/benchmarks.md`

---

## C) Core Language Surface Gaps (Spec vs Executable Surface)

### C-00a (P1) Canonical text UX: `Str` methods are the public surface
- [ ] Lock the canonical, user-facing text API to method-style `Str` calls (one way to write real code).
- **Why**: if we document/ship multiple competing styles (methods vs `std.text.*`), users will fork conventions and docs will drift.
- **Policy**:
  - The book (`book/`) and examples (`examples/`) must use **method-style** text operations (`raw.trim().to_lower()`, `s.contains(x)`, etc.).
  - If a limitation/transition exists (missing method parity, runtime gaps, naming changes), track it here as an actionable item instead of adding narrative disclaimers inside chapters.
  - Low-level primitives under `std.text` may exist, but are treated as internal/advanced reference—not the primary UX.
- **Acceptance**:
  - Book chapters do not include `std.text.*` call-style examples.
  - Appendix/reference docs do not present `std.text.*` as an “alternate style” for app code.
  - Any missing method parity vs text primitives is explicitly tracked as a checklist item (with example coverage).

### C-00 (P0) Data-modeling direction lock (types-first)
- [x] Lock and publish the canonical data-modeling direction:
  - first-class nominal `type` declarations are the primary model for related mixed-type data
  - composition-first behavior reuse is the default
  - inheritance/class model remains explicitly gated by C-03 decision
  - structural "shape"-style modeling (if adopted) is boundary-focused, not a replacement for nominal types
- **Why**: avoid map-record drift and keep language semantics consistent with production expectations.
- **Acceptance**:
  - Direction is documented in spec docs and mirrored in examples policy
  - `examples/11_modeling_shapes/` notes are aligned to this direction

### C-01 (P0) User-defined type declarations (struct-like shapes)
- [x] Implement end-to-end executable support for `type` declarations with mixed field types (MVP: heap records, 8-byte slots).
- **Why**: Required for real struct/shape modeling parity (C++/Rust-like workflows).
- **Evidence**:
  - Spec syntax exists: `docs/spec/syntax.md` (`type` declaration section)
  - AST/parser declaration path remains function-centric:
    - `crates/vibe_ast/src/lib.rs`
    - `crates/vibe_parser/src/lib.rs`
- **Acceptance**:
  - Support mixed field types (`Int`, `Str`, `Bool`, `List<T>`, `Map<K,V>`, optional `T?`)
  - Typed field read/update with compile-time diagnostics on unknown/mismatched fields
  - Add runnable examples replacing map-record stand-ins in `examples/11_modeling_shapes/`

### C-01a (P0) Type construction and update ergonomics
- [x] Define and implement canonical construction/update ergonomics for `type` values (MVP: `Type { field: expr }`, `obj.field`, `obj.field = expr`).
- **Why**: production use needs concise, safe creation/update patterns (not map-style emulation).
- **Acceptance**:
  - One canonical constructor model documented (literal-style and/or constructor function)
  - Deterministic diagnostics for missing required fields and invalid field assignments
  - At least two end-to-end runnable examples (simple domain model + nested model)

### C-02 (P0) Enum and executable `match` support
- [x] Add enum syntax + value model + type checking + match lowering + exhaustiveness (MVP: no-payload enums, tag-based match).
- **Why**: Pattern matching parity and algebraic data modeling.
- **Evidence**:
  - `docs/spec/control_flow.md` and `docs/spec/type_system.md` reference match/exhaustiveness
  - Implementation status remains partial/deferred in spec coverage docs
- **Acceptance**:
  - Runnable enum/match examples and integration tests

### C-03 (P1) Inheritance/class model decision
- [ ] Decide and document whether inheritance is in-scope.
- **Options**:
  - Adopt inheritance and implement
  - Declare explicit non-goal and formalize composition-first model
- **Acceptance**:
  - Decision reflected in spec, checklist, and examples
  - If non-goal: add explicit migration guidance from inheritance patterns to composition patterns

### C-04 (P1) Trait/interface polymorphism
- [ ] Define and implement trait/interface MVP, or formally defer with migration guidance.
- **Evidence**: deferred notes in `docs/spec/type_system.md`

### C-04a (P1) Composition-first patterns as first-class guidance
- [ ] Publish normative examples for capability composition with nominal types.
- **Why**: if inheritance is deferred/non-goal, composition guidance must be explicit and runnable.
- **Acceptance**:
  - Add composition patterns using real `type` declarations (not map stand-ins)
  - Include at least one "policy + core model" example and one "pipeline composition" example

### C-05a (P1) Optional structural shape support (boundary payloads)
- [ ] Decide whether structural "shape" typing is needed for external payload boundaries.
- **Why**: users may want PHP/TS-like shape ergonomics for request/response/input schemas.
- **Acceptance**:
  - If adopted: specify syntax, typing rules, and relation to nominal `type`
  - If deferred/non-goal: document recommended nominal-type alternative and conversion patterns

### C-05 (P1) `mut` / `const` / optional ergonomics
- [ ] Close parser/typechecker/runtime gaps for explicit mutability and optional value semantics.
- **Why**: ergonomic parity and clear data-flow intent in public APIs.

### C-06 (P1) Generic container support beyond current freeze
- [ ] Expand executable support for `List<T>` / `Map<K,V>` combinations beyond narrow runtime freeze combinations.
- **Evidence**: deferred notes in `docs/spec/containers.md`

### C-07 (P2) Numeric width fidelity (`i32`, `u64`, `f64`) in executable surface
- [ ] Implement first-class width-aware numeric behavior and conversion checks.

---

## F) Production Standard Library + Boundary APIs (Missing for real apps)

These items cover the “everyday surfaces” required to build robust services/CLIs/jobs in VibeLang
without rewriting core functionality in another language.

### F-01 (P0) General JSON value model + `json.parse` / `json.stringify`
- [x] Implement a first-class JSON value type and general parse/stringify APIs.
- **Evidence**:
  - Runtime + lowering: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Conformance: `crates/vibe_cli/tests/phase12_stdlib.rs`
  - Example: `examples/07_stdlib_io_json_regex_http/47_json_parse_stringify_and_codecs.yb`
- **Why**: production apps need to consume/emit JSON payloads (configs, HTTP APIs, logs) without
  bespoke parsers.
- **Delivery status**: preview implementation shipped as canonicalized string APIs:
  `json.parse(Str) -> Str` and `json.stringify(Str) -> Str`; malformed parse returns `""` and
  never panics. Additionally, `json.from_map(Map<Str, Str>) -> Str` ships with smart type
  detection (integer/boolean values emitted unquoted).
- **Spec hooks**:
  - `docs/spec/containers.md` (maps/lists semantics used by JSON trees)
  - `docs/spec/strings_and_text.md` (string encoding/escapes)
  - `docs/spec/error_model.md` (Result/error conventions)
  - `docs/spec/cost_model.md` (allocation visibility expectations)
- **Acceptance**:
  - Add type `Json` (or canonical equivalent) representing: null/bool/number/string/array/object.
  - Add `json.parse(raw: Str) -> Result<Json, JsonError>` (no sentinel return values).
  - Add `json.stringify(value: Json) -> Str` with deterministic output (stable ordering policy if
    object ordering is normalized).
  - Ensure malformed input never panics; errors are surfaced via `Result`.
  - Add conformance tests for escapes, unicode, large payloads, nested structures.
- **Example (target ergonomics)**:
  - `val := json.parse(payload)?`
  - `name := val.get("name")?.as_str()?` (exact accessor API may differ, but MUST be safe)
  - `out := json.stringify(val)`

### F-02 (P0) Typed JSON encode/decode for nominal `type` values (boundary payloads)
- [x] Provide a canonical way to convert between JSON and user-defined nominal types.
- **Evidence**:
  - Canonical generated entrypoints: `json.encode_<Type>` / `json.decode_<Type>(raw, fallback)`
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Example coverage: `compiler/tests/fixtures/stdlib/json/basic.yb`, `crates/vibe_cli/tests/phase12_stdlib.rs`
  - Example: `examples/07_stdlib_io_json_regex_http/47_json_parse_stringify_and_codecs.yb`
- **Why**: production APIs should not be written as ad-hoc `Map<Str, ...>` plumbing; they need
  safe, typed request/response models.
- **Dependencies**: `C-01`/`C-01a` nominal types (done); canonical approach selected:
  compiler-generated codec entrypoints.
- **Spec hooks**:
  - `docs/spec/type_system.md` (conversion policy)
  - `docs/spec/module_and_visibility.md` (public API surfaces)
  - `docs/spec/abi_and_ffi.md` (if reflection/metadata is introduced)
- **Acceptance**:
  - Define one canonical approach (traits/derives/codegen) and document it.
  - `json.decode<T>(raw: Str) -> Result<T, JsonError>`
  - `json.encode<T>(value: T) -> Json` and/or `json.stringify<T>(value: T) -> Str`
  - Deterministic field mapping policy (naming, missing fields, unknown fields, optional fields).
- **Example (target ergonomics)**:
  - `type User { id: Int, name: Str, email: Str? }`
  - `user := json.decode<User>(payload)?`
  - `resp := json.stringify(user)`

### F-03 (P0) HTTP client (real requests) with explicit effects + timeouts
- [x] Implement an HTTP client surface suitable for production service-to-service calls.
- **Why**: “make HTTP requests” is table-stakes; protocol helpers alone are not enough.
- **Current state**: `std.http` now exposes sync request APIs:
  `request/request_status/get/post` with timeout and redirect handling.
- **Evidence**:
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Deterministic local-server tests: `crates/vibe_cli/tests/phase12_stdlib.rs`
  - Example: `examples/07_stdlib_io_json_regex_http/48_http_sync_client_unreachable_smoke.yb`
- **Delivery status**: shipped with sync request helpers and explicit `net` effect;
  structured request/response object types are deferred.
- **Spec hooks**:
  - `docs/spec/async_await_and_threads.md` (async model if client is async)
  - `docs/spec/concurrency_and_scheduling.md` (go/channel/select if streaming)
  - `docs/spec/error_model.md` (retryable vs fatal errors)
  - `docs/spec/cost_model.md` (buffering vs streaming)
- **Acceptance**:
  - `http.request(req: http.Request) -> Result<http.Response, http.Error>` (or equivalent)
  - First-class support for: timeouts, headers, status, body (bytes/stream), redirects policy.
  - Explicit effect annotation required (network is never “pure”).
  - Deterministic tests via local in-process server and golden responses.
- **Example (target ergonomics)**:
  - `req := http.Request { method: "GET", url: "https://api.x/y", timeout_ms: 2000 }`
  - `resp := http.request(req)?`
  - `if resp.status == 200 { println(resp.body_text()) }`

### F-04 (P0) Networking foundation in stdlib (`net`): TCP + DNS (+ TLS plan)
- [x] Move networking primitives out of `bench.*` gating into a real stdlib module surface.
- **Why**: production HTTP requires stable socket/DNS; keeping these as benchmark-only blocks real
  application development.
- **Current state**: `std.net` is active with `listen/listener_port/accept/connect/read/write/close/resolve`.
- **Evidence**:
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Conformance: `crates/vibe_cli/tests/phase12_stdlib.rs`
  - Module docs (TLS plan documented): `stdlib/net/README.md`
  - Example: `examples/07_stdlib_io_json_regex_http/49_net_listen_and_resolve_smoke.yb`
- **Spec hooks**:
  - `docs/spec/async_await_and_threads.md` and `docs/spec/concurrency_and_scheduling.md`
  - `docs/spec/unsafe_escape_hatches.md` (if raw sockets need unsafe paths)
- **Acceptance**:
  - `net.tcp_connect(host: Str, port: Int) -> Result<net.TcpConn, net.Error>`
  - `net.tcp_listen(...) -> Result<net.TcpListener, net.Error>`
  - `net.resolve(host: Str) -> Result<List<net.IpAddr>, net.Error>`
  - Define TLS story explicitly: either (a) first-class `net.TlsConn`, or (b) documented v1
    deferral with an approved interop boundary.

### F-05 (P0) Conversion + parsing surface (no sentinel values; consistent naming)
- [x] Provide production-grade conversions and parsing helpers aligned to spec conversion policy.
- **Why**: teams need predictable, explicit conversions (`Str`↔number, widths, bool) with good
  diagnostics and without “0 on failure” footguns.
- **Spec hooks**:
  - `docs/spec/type_system.md` (“lossy must be explicit”)
  - `docs/spec/numeric_model.md` (rounding/overflow/trap/saturate rules)
  - `docs/spec/error_model.md` (Result conventions)
- **Acceptance**:
  - Replace/augment sentinel-return parsers (ex: `json.parse_i64` returning `0`) with
    `Result`-returning equivalents (keep old APIs only if clearly deprecated and documented).
  - Canonical set (names illustrative; final names must be consistent):
    - `parse_i64(Str) -> Result<Int, ParseError>`
    - `parse_u64(Str) -> Result<u64, ParseError>`
    - `parse_f64(Str) -> Result<f64, ParseError>`
    - `to_str(x) -> Str` for primitive numeric/bool types (deterministic formatting)
  - Define and document cast syntax (`as` or equivalent) if supported, including overflow policy.
- **Example (target ergonomics)**:
  - `port := parse_u16(port_str)?`
  - `n := parse_i64(raw)?`
  - `s := to_str(n)`
- **Evidence**:
  - Canonical preview APIs shipped under `convert.*`: `to_int/parse_i64/to_float/parse_f64/to_str/to_str_f64`
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Fixtures/tests: `compiler/tests/fixtures/stdlib/convert/basic.yb`, `crates/vibe_cli/tests/phase12_stdlib.rs`
  - Example: `examples/07_stdlib_io_json_regex_http/50_convert_parsing_and_formatting.yb`
- **Delivery status**: shipped in preview with deterministic formatting and sentinel parse failures;
  Result-based parse/cast promotion is tracked as follow-up hardening.

### F-06 (P1) String/text “daily use” surface (split/trim/contains/replace; byte vs scalar rules)
- [x] Provide a minimal, coherent text API set that matches the spec’s string model.
- **Why**: production code needs standard text ops; without them, teams write inconsistent helpers.
- **Spec hooks**:
  - `docs/spec/strings_and_text.md` (encoding, indexing/slicing)
- **Acceptance**:
  - Provide canonical functions/methods for: `trim`, `split`, `contains`, `starts_with`,
    `ends_with`, `replace`, `to_lower`/`to_upper` (unicode policy must be explicit).
  - Clarify and enforce length semantics (byte length vs scalar length) to avoid surprises.
  - Add spec examples demonstrating safe slicing/indexing boundaries.
- **Evidence**:
  - APIs: `text.trim/contains/starts_with/ends_with/replace/to_lower/to_upper/byte_len/split_part`
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Docs + fixtures: `stdlib/text/README.md`, `compiler/tests/fixtures/stdlib/text/basic.yb`
  - Example: `examples/07_stdlib_io_json_regex_http/51_text_utilities_daily_ops.yb`

### F-07 (P1) Bytes/encoding utilities (hex/base64/urlencode) for APIs
- [x] Add byte/encoding helpers needed for web/service integrations.
- **Why**: HTTP APIs frequently require base64, hex, and URL encoding.
- **Spec hooks**:
  - `docs/spec/strings_and_text.md`
  - `docs/spec/cost_model.md` (allocation/copy visibility)
- **Acceptance**:
  - `encoding.hex_encode(bytes)`, `encoding.hex_decode(str)`
  - `encoding.base64_encode(bytes)`, `encoding.base64_decode(str)`
  - `encoding.url_encode(str)`, `encoding.url_decode(str)` (or HTTP-scoped location)
  - Clear error model (`Result`) and fuzz/property tests for roundtrips.
- **Evidence**:
  - APIs: `encoding.hex_encode/hex_decode/base64_encode/base64_decode/url_encode/url_decode`
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Fixtures/tests: `compiler/tests/fixtures/stdlib/encoding/basic.yb`, `crates/vibe_cli/tests/phase12_stdlib.rs`
  - Example: `examples/07_stdlib_io_json_regex_http/52_encoding_roundtrip_basics.yb`
- **Delivery status**: shipped in preview; decode errors currently follow sentinel-return behavior.

### F-08 (P1) Time for production: monotonic clock + parsing/formatting policy
- [x] Expand `std.time` to support production timeouts/metrics and consistent formatting.
- **Why**: wall-clock time is insufficient for timeouts; services need monotonic time.
- **Spec hooks**:
  - `docs/spec/error_model.md` (parsing failures)
  - `docs/spec/cost_model.md`
- **Acceptance**:
  - `time.monotonic_now_ms()` (or equivalent) for elapsed durations/timeouts.
  - Explicit time format policy (RFC3339/ISO8601) for parse/format, with tests.
- **Evidence**:
  - API: `time.monotonic_now_ms()`
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Docs/test coverage: `stdlib/time/README.md`, `crates/vibe_cli/tests/phase12_stdlib.rs`
  - Example: `examples/07_stdlib_io_json_regex_http/53_time_monotonic_smoke.yb`

### F-09 (P1) Logging/telemetry primitives with explicit effects
- [x] Provide a small, stable logging surface (and hooks for structured telemetry).
- **Why**: production requires observability; apps should not invent ad-hoc logging APIs.
- **Spec hooks**:
  - `docs/spec/error_model.md` (error formatting)
  - `docs/spec/module_and_visibility.md` (public API norms)
- **Acceptance**:
  - `log.info/warn/error(...)` with structured fields policy (or a single structured event API).
  - Clear effect requirement (logging is an effect).
  - Deterministic tests for formatting (and redaction policy if supported).
- **Evidence**:
  - APIs: `log.info/warn/error`
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Fixtures: `compiler/tests/fixtures/stdlib/log/basic.yb`
  - Example: `examples/07_stdlib_io_json_regex_http/54_log_primitives_smoke.yb`

### F-10 (P1) Env/config/CLI args surface (robust apps without bespoke glue)
- [x] Provide standard APIs for environment variables, argv parsing, and exit codes.
- **Why**: CLIs and services need config; production teams expect these utilities.
- **Spec hooks**:
  - `docs/spec/error_model.md`
  - `docs/spec/module_and_visibility.md`
- **Acceptance**:
  - `env.get(key) -> Str` and `env.get_required(key) -> Str` (preview sentinel model)
  - `cli.args_len() -> Int`, `cli.arg(index) -> Str` (canonical equivalent for current runtime model)
  - A documented config loading pattern (env + file + defaults) with examples.
- **Evidence**:
  - APIs: `env.get/has/get_required`, `cli.args_len/arg`
  - Runtime + lowering + typing: `runtime/native/vibe_runtime.c`, `crates/vibe_codegen/src/lib.rs`, `crates/vibe_types/src/lib.rs`
  - Docs + fixtures: `stdlib/env/README.md`, `stdlib/cli/README.md`, `compiler/tests/fixtures/stdlib/env_cli/basic.yb`
  - Example: `examples/07_stdlib_io_json_regex_http/55_env_cli_surface_smoke.yb`

### F-11 (P0) `text.index_of` -- substring position search
- [x] Add `text.index_of(haystack: Str, needle: Str) -> Int` to the text module.
- **Why**: production HTML/text parsing requires finding the byte offset of substrings, not just
  boolean containment. This is a blocking gap for building real parsers in VibeLang.
- **Evidence**:
  - Runtime: `runtime/native/vibe_runtime.c` (`vibe_text_index_of`)
  - Codegen: `crates/vibe_codegen/src/lib.rs` (`text_index_of_fn`)
  - Typing: `crates/vibe_types/src/lib.rs` (`("text", "index_of")`)
  - Docs: `stdlib/text/README.md`
  - Example: `examples/07_stdlib_io_json_regex_http/56_text_index_of_basics.yb`
  - Tests: `crates/vibe_cli/tests/phase12_stdlib.rs` (existing surface test passes)
- **Acceptance**:
  - Returns byte offset of first occurrence, or `-1` if not found.
  - Empty needle returns `0`.
  - Non-panicking for arbitrary input (NULL-safe in C runtime).

### F-12 (P0) `Map<Str, Str>` + `json.from_map` -- dict-like JSON construction
- [x] Add `Map<Str, Str>` container support (runtime, codegen, type checker).
- [x] Add `json.from_map(map: Map<Str, Str>) -> Str` with smart type detection.
- **Why**: production HTTP services need to construct JSON responses without manual
  `str_builder.append` calls for every field. `Map<Str, Str>` + `json.from_map` gives
  Python-like `dict` + `json.dumps` ergonomics.
- **Evidence**:
  - Runtime: `runtime/native/vibe_runtime.c` (`vibe_map_str_str`, `vibe_json_from_str_str_map`)
  - Codegen: `crates/vibe_codegen/src/lib.rs` (`map_new_str_str_fn`, `container_set_str_str_fn`, `json_from_str_str_map_fn`)
  - Typing: `crates/vibe_types/src/lib.rs` (json.from_map special handling)
  - Docs: `stdlib/json/README.md`
  - Example: `examples/07_stdlib_io_json_regex_http/58_map_str_str_json_from_map.yb`
- **Acceptance**:
  - `{"key": "value"}` literal creates `Map<Str, Str>` when all values are strings.
  - `json.from_map(m)` emits JSON with auto type detection: integer strings unquoted, "true"/"false" as booleans, others as quoted strings.
  - Insertion-order preserved in JSON output.

### F-13 (P0) HTTPS curl byte-count fix
- [x] Fix off-by-one byte counts in `vibe_http_request_body_curl` and `vibe_http_request_status_curl`.
- **Why**: HTTPS requests via curl were silently failing because the `--max-time` parameter
  was truncated by a null byte in the command string.
- **Evidence**: `runtime/native/vibe_runtime.c` (corrected string lengths: 23, 15, 12)
- **Acceptance**: `http.get("https://...", timeout)` returns response body for HTTPS URLs.

---

## D) Example Program Quality Gates (Required Before “Production-Ready Examples” Claim)

### D-01 (P1) CI static check sweep
- [x] Add CI job: run `vibe check` on all examples.

### D-02 (P1) CI runtime sweep for runnable entries
- [x] Add CI job: run `vibe run` on all non-demo entry examples.

### D-03 (P1) Intentional-failure allowlist governance
- [x] Maintain explicit allowlist of intentional-failure demo examples.

### D-04 (P1) Checklist-ID linkage for failures
- [x] Require every failing example to reference a checklist ID from this file.

### D-05 (P1) Parity trend reporting
- [x] Publish periodic example parity report (pass/fail trend).

---

## E) Recommended Execution Order

1. **P0 runtime/codegen parity**: A-01..A-07
2. **P0 language data-modeling core**: C-00, C-01, C-01a, C-02
3. **P1 benchmark strict-publication blockers**: B-01..B-04
4. **P0/P1 production stdlib boundary surface**: F-01..F-05 (then F-06..F-10)
5. **P1 modeling decisions and ergonomics**: C-03, C-04, C-04a, C-05, C-05a, C-06
6. **P2 optimization and advanced surface**: B-05, C-07

This order ensures semantic examples can remain “truthful” while the implementation catches up without workaround drift.
