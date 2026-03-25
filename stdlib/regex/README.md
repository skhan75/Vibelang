# `regex` module (preview)

## APIs

- `regex.count(text: Str, pattern: Str) -> Int`
- `regex.replace_all(text: Str, pattern: Str, replacement: Str) -> Str`

## Semantics

- `count` returns the number of non-overlapping matches of `pattern` in `text`.
- `replace_all` replaces all non-overlapping matches of `pattern` in `text` with `replacement` and returns the resulting string.

## Error model

- Invalid patterns return `0` for `count` and the original `text` for `replace_all`.
- APIs are non-panicking for arbitrary input.
