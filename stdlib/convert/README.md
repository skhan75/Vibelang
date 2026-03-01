# `convert` module (preview)

## APIs

- `convert.to_int(raw: Str) -> Int`
- `convert.parse_i64(raw: Str) -> Int`
- `convert.to_float(raw: Str) -> Float`
- `convert.parse_f64(raw: Str) -> Float`
- `convert.to_str(value: Int) -> Str`
- `convert.to_str_f64(value: Float) -> Str`

## Semantics

- `to_int/parse_i64` parse strict decimal integer text with surrounding whitespace support.
- `to_float/parse_f64` parse strict floating-point text with surrounding whitespace support.
- `to_str` formats integers in canonical decimal form.
- `to_str_f64` uses shortest-roundtrip formatting for deterministic string output.

## Error model

- Parsing APIs are sentinel-return in this preview tier:
  - invalid integer/float input returns `0` / `0.0`.
- Formatting APIs are non-panicking for all valid numeric inputs.
- Migration path to Result-based parsing is tracked in the canonical checklist:
  `docs/checklists/features_and_optimizations.md` (`F-05`).
