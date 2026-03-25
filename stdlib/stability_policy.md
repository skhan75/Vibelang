# Stdlib Stability Policy (Phase 12)

This policy defines compatibility expectations for the expanded stdlib surface.

## Tiers

- **stable**
  - source/API compatibility expected across `v1.x`
  - behavior covered by deterministic tests and reference docs
  - breaking changes require major-version bump or explicit compatibility shim
- **preview**
  - intended for production trial use, but signature/semantics can still evolve in minor versions
  - any change requires release-note callout and migration guidance
- **experimental**
  - rapid iteration surface, may change without migration tooling
  - cannot be required by release-gate examples
- **internal**
  - implementation detail (`vibe_*` runtime symbols), no public compatibility guarantees

## Current Classification

- `io.print`, `io.println`: **stable**
- `path.join`, `path.parent`, `path.basename`, `path.is_absolute`: **stable**
- deterministic helpers in `core` (`len`, `min`, `max`, `sorted_desc`, `sort_desc`, `take`):
  **stable** for `vibe test` contract/example execution
- `time.now_ms`, `time.monotonic_now_ms`, `time.sleep_ms`, `time.duration_ms`: **preview**
- `fs.exists`, `fs.read_text`, `fs.write_text`, `fs.create_dir`: **preview**
- `net.listen`, `net.listener_port`, `net.accept`, `net.connect`, `net.read`, `net.write`, `net.close`, `net.resolve`: **preview**
- `convert.to_int`, `convert.parse_i64`, `convert.to_float`, `convert.parse_f64`, `convert.to_str`, `convert.to_str_f64`: **preview**
- `text.trim`, `text.contains`, `text.starts_with`, `text.ends_with`, `text.replace`, `text.to_lower`, `text.to_upper`, `text.byte_len`, `text.split_part`: **preview**
- `encoding.hex_encode`, `encoding.hex_decode`, `encoding.base64_encode`, `encoding.base64_decode`, `encoding.url_encode`, `encoding.url_decode`: **preview**
- `json.is_valid`, `json.parse`, `json.stringify`, `json.parse_i64`, `json.stringify_i64`, `json.minify`, typed codecs (`json.encode`, `json.decode`): **preview**
- `http.status_text`, `http.default_port`, `http.build_request_line`, `http.request`, `http.request_status`, `http.get`, `http.post`: **preview**
- `log.info`, `log.warn`, `log.error`: **preview**
- `env.get`, `env.has`, `env.get_required`: **preview**
- `cli.args_len`, `cli.arg`: **preview**
- runtime bridge symbols (`vibe_*` C ABI): **internal**

## Change Rules

- Stable APIs:
  - no breaking signature changes in `v1.x`
  - semantic changes require migration notes + compatibility statement
- Preview APIs:
  - may change in minor versions
  - must update docs + deterministic tests + release notes in the same change
- Experimental APIs:
  - no compatibility promises
  - cannot silently graduate; promotion requires explicit tier update in this file

## Determinism and error-model requirements

- Stable and preview APIs must document:
  - deterministic behavior expectations
  - error model (`panic`, sentinel return, or explicit boolean/result contract)
- Non-deterministic APIs (currently `time.now_ms`, `time.monotonic_now_ms`, `env.*`, `cli.*`)
  must be explicitly marked.
