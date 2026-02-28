# `http` module (preview)

## APIs

- `http.status_text(code: Int) -> Str`
- `http.default_port(scheme: Str) -> Int`
- `http.build_request_line(method: Str, path: Str) -> Str`
- `http.server_bench(n: Int) -> Int`

## Semantics

- `status_text` maps common status codes (`200`, `201`, `204`, `400`, `401`, `403`, `404`,
  `500`) and returns `"Unknown"` otherwise.
- `default_port` returns `443` for `https`/`wss`, otherwise `80`.
- `build_request_line` emits canonical `METHOD PATH HTTP/1.1`.
- `server_bench` runs a local HTTP server/client microbenchmark and returns the sum of response
  values. This exists to support third-party benchmark parity.

## Scope

Phase 12 exposes HTTP protocol essentials only. Full client/server transport stacks remain out of
scope for this surface and will be tracked in later phases.
