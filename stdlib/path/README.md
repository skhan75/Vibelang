# `path` module (stable)

## APIs

- `path.join(base: Str, leaf: Str) -> Str`
- `path.parent(value: Str) -> Str`
- `path.basename(value: Str) -> Str`
- `path.is_absolute(value: Str) -> Bool`

## Semantics

- `join` inserts `/` when needed and preserves existing separators.
- `parent` returns:
  - `/` for root,
  - `.` when there is no parent component.
- `basename` returns the final segment after `/` or `\`.
- `is_absolute` accepts Unix absolute paths and drive-letter style prefixes.

## Error model

- Functions are non-panicking for empty input.
- Empty/null-like cases resolve to deterministic fallback strings (`""`, `.`, or `/`).
