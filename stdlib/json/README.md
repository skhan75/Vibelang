# `json` module (preview)

The module centers on the `Json` value type: parse text into `Json`, stringify `Json` back to text, and build JSON incrementally with `json.builder` when structure is dynamic. Typed boundaries use `json.encode` / `json.decode` with compiler-inferred types.

**Recommended paths**

- **Arbitrary / runtime-shaped JSON**: `json.parse` → `Json`, then `json.stringify` / `json.stringify_pretty`; or build with `json.builder.*` and `json.builder.finish`.
- **Fixed nominal models**: `json.encode(value)` / `json.decode(raw, fallback)`.
- **Legacy convenience**: `json.from_map` (string values only, heuristic typing) — not the primary API.

`Result`-based JSON errors and richer typed codec surfaces are future work; today errors are panic-or-sentinel as noted below.

## APIs

**`Json` lifecycle**

- `json.parse(raw: Str) -> Json` — strict parse; **panics** on invalid JSON
- `json.stringify(value: Json) -> Str` — compact deterministic output
- `json.stringify_pretty(value: Json) -> Str` — pretty-printed output

**`Json` constructors**

- `json.null() -> Json`
- `json.bool(value: Bool) -> Json`
- `json.i64(value: Int) -> Json`
- `json.f64(value: Float) -> Json`
- `json.str(value: Str) -> Json`

**Incremental builder (`JsonBuilder`)**

- `json.builder.new(capacity: Int) -> JsonBuilder`
- `json.builder.begin_object(builder: JsonBuilder) -> JsonBuilder`
- `json.builder.end_object(builder: JsonBuilder) -> JsonBuilder`
- `json.builder.begin_array(builder: JsonBuilder) -> JsonBuilder`
- `json.builder.end_array(builder: JsonBuilder) -> JsonBuilder`
- `json.builder.key(builder: JsonBuilder, name: Str) -> JsonBuilder`
- `json.builder.value_null(builder: JsonBuilder) -> JsonBuilder`
- `json.builder.value_bool(builder: JsonBuilder, value: Bool) -> JsonBuilder`
- `json.builder.value_i64(builder: JsonBuilder, value: Int) -> JsonBuilder`
- `json.builder.value_f64(builder: JsonBuilder, value: Float) -> JsonBuilder`
- `json.builder.value_str(builder: JsonBuilder, value: Str) -> JsonBuilder`
- `json.builder.value_json(builder: JsonBuilder, value: Json) -> JsonBuilder`
- `json.builder.finish(builder: JsonBuilder) -> Str`

**Typed codecs (type-inferred)**

- `json.encode(value) -> Str` — the compiler infers the struct type from the argument
- `json.decode(raw: Str, fallback) -> T` — the compiler infers the target type from the fallback argument

Nested struct fields are recursively encoded/decoded. A `type Outer { inner: Inner }` where `Inner` is also a user-defined struct produces nested JSON objects automatically.

The legacy `json.encode_<Type>` / `json.decode_<Type>` syntax with explicit type suffix is still accepted for backward compatibility.

**Compatibility / utilities**

- `json.from_map(map: Map<Str, Str>) -> Str` — convenience only; all map values are strings; see semantics
- `json.is_valid(raw: Str) -> Bool`
- `json.try_parse(raw: Str) -> Result<Json, Str>` — non-aborting parse for network bytes
- `json.result_value(parsed: Result<Json, Str>) -> Json` — the document, or a JSON `null` node on `Err`
- `json.result_error(parsed: Result<Json, Str>) -> Str` — the message, or `""` on `Ok`
- `json.parse_i64(raw: Str) -> Int`
- `json.stringify_i64(value: Int) -> Str`
- `json.minify(raw: Str) -> Str`

## Semantics

- **`parse` / `stringify` / `stringify_pretty`** operate on the `Json` AST: escapes and structure follow normal JSON rules. Output is deterministic for a given `Json` value.
- **`json.builder`**: emit JSON by nesting `begin_object` / `end_object`, `begin_array` / `end_array`, `key` (in objects), then scalar/`value_json` calls. **`finish`** produces the final `Str`; invalid sequencing or misuse can **panic** (same spirit as `parse` strictness).
- **`encode` / `decode`**: the compiler resolves the struct type from the argument at compile time and generates the appropriate codec. Field mapping is deterministic for supported field types (`Int`, `Str`, `Bool`, `Json`, and nested user-defined struct types). Nested structs are recursively encoded to JSON objects and recursively decoded from JSON objects. **`decode`** uses **`fallback`** for missing or invalid fields.
- **`from_map`**: serializes `Map<Str, Str>` to a JSON object. Values are still strings at the type level; runtime applies heuristics: integer-looking values unquoted, `"true"` / `"false"` as booleans, otherwise JSON strings. Prefer **`json.builder`** or **`Json`** + **`stringify`** when you need explicit types without guessing.
- **`is_valid`**: runs the real grammar in validate-only mode — the same parser code, with node construction skipped — so it agrees with `parse` exactly on grammar, trailing content and nesting depth while allocating nothing per node; returns `false` for malformed input. Guarantee: `is_valid(s) == true` implies `parse(s)` returns rather than aborting. (It used to compare only the first and last non-space characters, which made it answer `true` for `[x]` and `[1,2,]`.)
- **`try_parse`**: same grammar as `parse`, but returns `Ok(Json)` / `Err(Str)` instead of aborting. Total with respect to input — no byte sequence reaches a panic through it. Read the outcome with `result.is_ok` / `result.is_err` plus `json.result_value` / `json.result_error` (`result.unwrap_or` supports `Result<Int, _>` only).
- **Nesting depth**: `parse`, `try_parse` and `is_valid` share a 256-level cap. The parser is recursive, so without the cap deeply nested text exhausts the thread stack and terminates the process by signal. Past the cap `parse` panics with a distinct message and `try_parse` returns `Err`.
- **`parse_i64`**: parses integer literals with surrounding whitespace.
- **`stringify_i64`**: decimal string for `Int`.
- **`minify`**: drops insignificant whitespace while preserving string contents and escapes; intended for JSON text.

## Benchmark-only helpers

Some benchmark parity helpers were intentionally moved out of the default stdlib surface. See `stdlib/bench/README.md` for:

- `bench.json_canonical`
- `bench.json_repeat_array`

## Error model

- **`json.parse`**: invalid JSON → **panic** (no sentinel `Json`); nesting past 256 levels → **panic** with a distinct message. Use **`json.try_parse`** for text that arrived from a socket.
- **`json.try_parse`**: never panics for any input; malformed or over-deep input → **`Err(Str)`**.
- **`json.stringify` / `json.stringify_pretty`**: serialize a `Json` value; the runtime maps a null handle to **`""`** (defensive), which the typed surface should not normally produce.
- **`json.builder.finish`** / mismatched **`begin_*` / `end_*`**: **panic** on misuse.
- **`json.is_valid`**: `false` for malformed input; non-panicking.
- **`json.parse_i64`**: returns **`0`** for invalid numeric input.
- **`json.decode`**: uses provided **`fallback`** for recoverable decode issues (per generated codec behavior).
- **`json.minify`**: non-panicking for arbitrary text input (best-effort minification).
