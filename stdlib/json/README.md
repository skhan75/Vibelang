# `json` module (preview)

## APIs

- `json.is_valid(raw: Str) -> Bool`
- `json.parse_i64(raw: Str) -> Int`
- `json.stringify_i64(value: Int) -> Str`
- `json.minify(raw: Str) -> Str`
- `json.canonical(raw: Str) -> Str`
- `json.repeat_array(item: Str, n: Int) -> Str`

## Semantics

- `is_valid` supports common JSON literals and basic structural wrappers (`{}`, `[]`, quoted
  strings, booleans, null, integer literals).
- `parse_i64` parses integer literals with surrounding whitespace.
- `stringify_i64` serializes `Int` to canonical decimal string.
- `minify` removes insignificant whitespace while preserving string literals and escapes.
- `canonical` emits a compact canonical JSON string, normalizing numeric formatting to match the
  implementation’s canonical serializer behavior.
- `repeat_array` constructs a compact JSON array by repeating `item` \(n\) times: `[...]`.

## Error model

- `parse_i64` returns `0` for invalid numeric input.
- `is_valid` returns `false` for malformed input.
- `minify` is non-panicking for arbitrary text input.
