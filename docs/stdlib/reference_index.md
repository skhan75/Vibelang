# Stdlib Reference Index (Phase 12)

This index is the authoritative entry point for Phase 12 stdlib APIs.

## Modules

- `io` (stable): printing primitives
- `core` (stable): deterministic utilities for examples/contracts
- `time` (preview): `now_ms`, `monotonic_now_ms`, `sleep_ms`, `duration_ms`
- `path` (stable): `join`, `parent`, `basename`, `is_absolute`
- `fs` (preview): `exists`, `read_text`, `write_text`, `create_dir`
- `net` (preview): `listen`, `listener_port`, `accept`, `connect`, `read`, `write`, `close`, `resolve`
- `convert` (preview): `to_int`, `parse_i64`, `to_float`, `parse_f64`, `to_str`, `to_str_f64`
- `text` (preview): `trim`, `contains`, `starts_with`, `ends_with`, `replace`, `index_of`, `to_lower`, `to_upper`, `byte_len`, `split_part`
- `encoding` (preview): `hex_encode`, `hex_decode`, `base64_encode`, `base64_decode`, `url_encode`, `url_decode`
- `json` (preview): `Json` — `parse` (strict), `stringify`, `stringify_pretty`, `null`/`bool`/`i64`/`f64`/`str`; `json.builder.*` (`new`, `begin_object`/`end_object`, `begin_array`/`end_array`, `key`, `value_*`, `finish`); `encode_<Type>`/`decode_<Type>`; `from_map` (compatibility); `is_valid`, `parse_i64`, `stringify_i64`, `minify`
- `http` (preview): `status_text`, `default_port`, `build_request_line`, `request`, `request_status`, `get`, `post`
- `log` (preview): `info`, `warn`, `error`
- `env` (preview): `get`, `has`, `get_required`
- `cli` (preview): `args_len`, `arg`
- `str_builder` (preview): `new`, `append`, `append_char`, `finish`
- `regex` (preview): `count`, `replace_all`

## Primary module docs

- `stdlib/README.md`
- `stdlib/stability_policy.md`
- `stdlib/time/README.md`
- `stdlib/path/README.md`
- `stdlib/fs/README.md`
- `stdlib/net/README.md`
- `stdlib/convert/README.md`
- `stdlib/text/README.md`
- `stdlib/str_builder/README.md`
- `stdlib/regex/README.md`
- `stdlib/encoding/README.md`
- `stdlib/json/README.md`
- `stdlib/http/README.md`
- `stdlib/log/README.md`
- `stdlib/env/README.md`
- `stdlib/cli/README.md`
