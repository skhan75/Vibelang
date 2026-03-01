# `http` module (preview)

## APIs

- `http.status_text(code: Int) -> Str`
- `http.default_port(scheme: Str) -> Int`
- `http.build_request_line(method: Str, path: Str) -> Str`
- `http.request(method: Str, url: Str, body: Str, timeout_ms: Int) -> Str`
- `http.request_status(method: Str, url: Str, body: Str, timeout_ms: Int) -> Int`
- `http.get(url: Str, timeout_ms: Int) -> Str`
- `http.post(url: Str, body: Str, timeout_ms: Int) -> Str`

## Semantics

- `status_text` maps common status codes (`200`, `201`, `204`, `400`, `401`, `403`, `404`,
  `500`) and returns `"Unknown"` otherwise.
- `default_port` returns `443` for `https`/`wss`, otherwise `80`.
- `build_request_line` emits canonical `METHOD PATH HTTP/1.1`.
- `request` performs a sync request and returns response body text.
- `request_status` performs the same request and returns only the status code.
- `get`/`post` are convenience wrappers over `request`.
- Transport behavior:
  - `http://` URLs use native TCP path.
  - `https://` URLs use TLS transport via system `curl` path in this milestone.
 
## Benchmark-only helpers

The HTTP server microbenchmark entrypoint is exposed under `bench.http_server_bench` (see
`stdlib/bench/README.md`) and is only available when Vibe is built with `--features bench-runtime`.

## Scope

This preview surface is sync-first and optimized for deterministic service-to-service flows
(timeouts + redirects + explicit status retrieval). Advanced streaming/client pooling will be
tracked in later phases.
