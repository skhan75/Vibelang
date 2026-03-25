# `text` module (preview)

## APIs

- `text.trim(raw: Str) -> Str`
- `text.contains(raw: Str, needle: Str) -> Bool`
- `text.starts_with(raw: Str, prefix: Str) -> Bool`
- `text.ends_with(raw: Str, suffix: Str) -> Bool`
- `text.replace(raw: Str, from: Str, to: Str) -> Str`
- `text.to_lower(raw: Str) -> Str`
- `text.to_upper(raw: Str) -> Str`
- `text.byte_len(raw: Str) -> Int`
- `text.split_part(raw: Str, sep: Str, index: Int) -> Str`
- `text.index_of(haystack: Str, needle: Str) -> Int`

## Semantics

- Operations are byte-oriented on UTF-8 text in this preview tier.
- `split_part` returns the `index`-th segment (0-based) and `""` when missing.
- `byte_len` is explicit byte length (not Unicode scalar count).
- `index_of` returns the byte offset of the first occurrence of `needle` in `haystack`, or `-1` if not found. Empty needle returns `0`.

## Error model

- APIs are non-panicking for arbitrary input.
- Out-of-range `split_part` returns `""`.
