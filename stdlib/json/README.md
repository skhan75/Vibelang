# `json` module (preview)

## APIs

- `json.is_valid(raw: Str) -> Bool`
- `json.parse(raw: Str) -> Str`
- `json.stringify(raw: Str) -> Str`
- `json.parse_i64(raw: Str) -> Int`
- `json.stringify_i64(value: Int) -> Str`
- `json.minify(raw: Str) -> Str`
- `json.encode_<Type>(value: Type) -> Str` (compiler-generated typed codec entrypoint)
- `json.decode_<Type>(raw: Str, fallback: Type) -> Type` (compiler-generated typed codec entrypoint)

## Semantics

- `is_valid` supports common JSON literals and basic structural wrappers (`{}`, `[]`, quoted
  strings, booleans, null, integer literals).
- `parse` returns canonicalized JSON text for valid JSON input and empty string for invalid input.
- `stringify` returns canonicalized JSON if input is valid JSON, otherwise emits a quoted JSON
  string value.
- `parse_i64` parses integer literals with surrounding whitespace.
- `stringify_i64` serializes `Int` to canonical decimal string.
- `minify` removes insignificant whitespace while preserving string literals and escapes.
- `encode_<Type>` and `decode_<Type>` are generated from nominal `type` declarations and currently
  support deterministic field mapping for `Int`, `Str`, and `Bool` fields.
 
## Benchmark-only helpers

Some benchmark parity helpers were intentionally moved out of the default stdlib surface. See
`stdlib/bench/README.md` for:

- `bench.json_canonical`
- `bench.json_repeat_array`

## Error model

- `parse_i64` returns `0` for invalid numeric input.
- `parse` returns `""` for invalid JSON input.
- `is_valid` returns `false` for malformed input.
- `minify` is non-panicking for arbitrary text input.
- `decode_<Type>` uses the provided `fallback` value for missing/invalid fields.
