# `cli` module (preview)

## APIs

- `cli.args_len() -> Int`
- `cli.arg(index: Int) -> Str`

## Semantics

- `args_len` returns number of positional arguments excluding executable name.
- `arg(index)` is 0-based and returns `""` when out-of-range.

## Platform notes

- Linux path reads `/proc/self/cmdline`.
- Non-Linux targets currently return empty/default values in this preview tier.
