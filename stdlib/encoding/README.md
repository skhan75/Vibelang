# `encoding` module (preview)

## APIs

- `encoding.hex_encode(raw: Str) -> Str`
- `encoding.hex_decode(raw: Str) -> Str`
- `encoding.base64_encode(raw: Str) -> Str`
- `encoding.base64_decode(raw: Str) -> Str`
- `encoding.url_encode(raw: Str) -> Str`
- `encoding.url_decode(raw: Str) -> Str`

## Error model

- Decode helpers return `""` on malformed input in this preview tier.
- Encode helpers are non-panicking for arbitrary string input.
