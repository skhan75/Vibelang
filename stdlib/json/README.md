# `json` module (preview)

The module centers on the `Json` value type: parse text into `Json`, stringify `Json` back to text, and build JSON incrementally with `json.builder` when structure is dynamic. Typed boundaries use compiler-generated `json.encode_<Type>` / `json.decode_<Type>`.

**Recommended paths**

- **Arbitrary / runtime-shaped JSON**: `json.parse` → `Json`, then `json.stringify` / `json.stringify_pretty`; or build with `json.builder.*` and `json.builder.finish`.
- **Fixed nominal models**: `json.encode_<Type>` / `json.decode_<Type>`.
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

**Typed codecs (compiler-generated)**

- `json.encode_<Type>(value: Type) -> Str`
- `json.decode_<Type>(raw: Str, fallback: Type) -> Type`

Nested struct fields are recursively encoded/decoded. A `type Outer { inner: Inner }` where `Inner` is also a user-defined struct produces nested JSON objects automatically.

**Compatibility / utilities**

- `json.from_map(map: Map<Str, Str>) -> Str` — convenience only; all map values are strings; see semantics
- `json.is_valid(raw: Str) -> Bool`
- `json.parse_i64(raw: Str) -> Int`
- `json.stringify_i64(value: Int) -> Str`
- `json.minify(raw: Str) -> Str`

## Semantics

- **`parse` / `stringify` / `stringify_pretty`** operate on the `Json` AST: escapes and structure follow normal JSON rules. Output is deterministic for a given `Json` value.
- **`json.builder`**: emit JSON by nesting `begin_object` / `end_object`, `begin_array` / `end_array`, `key` (in objects), then scalar/`value_json` calls. **`finish`** produces the final `Str`; invalid sequencing or misuse can **panic** (same spirit as `parse` strictness).
- **`encode_<Type>` / `decode_<Type>`**: generated from nominal `type` declarations; field mapping is deterministic for supported field types (`Int`, `Str`, `Bool`, and nested user-defined struct types). Nested structs are recursively encoded to JSON objects and recursively decoded from JSON objects. **`decode_*`** uses **`fallback`** for missing or invalid fields where implemented.
- **`from_map`**: serializes `Map<Str, Str>` to a JSON object. Values are still strings at the type level; runtime applies heuristics: integer-looking values unquoted, `"true"` / `"false"` as booleans, otherwise JSON strings. Prefer **`json.builder`** or **`Json`** + **`stringify`** when you need explicit types without guessing.
- **`is_valid`**: structural/literal validation without building a full `Json` value for the caller; returns `false` for malformed input.
- **`parse_i64`**: parses integer literals with surrounding whitespace.
- **`stringify_i64`**: decimal string for `Int`.
- **`minify`**: drops insignificant whitespace while preserving string contents and escapes; intended for JSON text.

## Benchmark-only helpers

Some benchmark parity helpers were intentionally moved out of the default stdlib surface. See `stdlib/bench/README.md` for:

- `bench.json_canonical`
- `bench.json_repeat_array`

## Error model

- **`json.parse`**: invalid JSON → **panic** (no sentinel `Json`).
- **`json.stringify` / `json.stringify_pretty`**: serialize a `Json` value; the runtime maps a null handle to **`""`** (defensive), which the typed surface should not normally produce.
- **`json.builder.finish`** / mismatched **`begin_*` / `end_*`**: **panic** on misuse.
- **`json.is_valid`**: `false` for malformed input; non-panicking.
- **`json.parse_i64`**: returns **`0`** for invalid numeric input.
- **`json.decode_<Type>`**: uses provided **`fallback`** for recoverable decode issues (per generated codec behavior).
- **`json.minify`**: non-panicking for arbitrary text input (best-effort minification).
