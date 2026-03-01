# `log` module (preview)

## APIs

- `log.info(message: Str) -> Void`
- `log.warn(message: Str) -> Void`
- `log.error(message: Str) -> Void`

## Semantics

- Emits prefixed line-based logs:
  - `info` -> `[info] ...`
  - `warn` -> `[warn] ...`
  - `error` -> `[error] ...` (stderr)

## Effects

- All logging APIs require `@effect io`.
