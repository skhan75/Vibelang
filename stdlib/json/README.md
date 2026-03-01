# `json` module (preview)

## APIs

- `json.is_valid(raw: Str) -> Bool`
- `json.parse_i64(raw: Str) -> Int`
- `json.stringify_i64(value: Int) -> Str`
- `json.minify(raw: Str) -> Str`

## Semantics

- `is_valid` supports common JSON literals and basic structural wrappers (`{}`, `[]`, quoted
  strings, booleans, null, integer literals).
- `parse_i64` parses integer literals with surrounding whitespace.
- `stringify_i64` serializes `Int` to canonical decimal string.
- `minify` removes insignificant whitespace while preserving string literals and escapes.
 
## Benchmark-only helpers

Some benchmark parity helpers were intentionally moved out of the default stdlib surface. See
`stdlib/bench/README.md` for:

- `bench.json_canonical`
- `bench.json_repeat_array`

## Error model

- `parse_i64` returns `0` for invalid numeric input.
- `is_valid` returns `false` for malformed input.
- `minify` is non-panicking for arbitrary text input.
