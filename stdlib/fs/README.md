# `fs` module (preview)

## APIs

- `fs.exists(path: Str) -> Bool`
- `fs.read_text(path: Str) -> Str`
- `fs.write_text(path: Str, contents: Str) -> Bool`
- `fs.create_dir(path: Str) -> Bool`

## Semantics

- `exists` checks local filesystem visibility for `path`.
- `read_text` reads entire file as UTF-8 bytes and returns empty string on failure.
- `write_text` writes full contents and returns `true` on success.
- `create_dir` creates one directory level and returns `true` if created or already present.

## Error model

- API surface uses sentinel boolean/empty-string returns instead of throwing/panicking for
  common I/O failures.
- Runtime allocation failures are fatal (global runtime policy).

## Effects

- All `fs.*` calls are `io` effects.
