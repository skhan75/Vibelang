# `env` module (preview)

## APIs

- `env.get(key: Str) -> Str`
- `env.has(key: Str) -> Bool`
- `env.get_required(key: Str) -> Str`

## Error model

- Missing keys return `""` from `get/get_required`.
- `has` returns `false` when key is absent or invalid.

## Effects

- Environment inspection is treated as `nondet`.
