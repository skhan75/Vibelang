# `http` module (preview)

## Types (auto-injected by compiler)

```vibelang
type HttpRequest {
  method: Str,
  url: Str,
  headers: Str,
  body: Str,
  timeout_ms: Int
}

type HttpResponse {
  status: Int,
  headers: Str,
  body: Str
}
```

`headers` uses raw HTTP format: `"Content-Type: application/json\r\nAuthorization: Bearer tok"`.

## Client API

- `http.send(req: HttpRequest) -> HttpResponse` — full-control client
- `http.get(url: Str, timeout_ms: Int) -> HttpResponse` — convenience GET
- `http.post(url: Str, body: Str, timeout_ms: Int) -> HttpResponse` — convenience POST

All client calls require `@effect net`.

## Server API

- `http.response(resp: HttpResponse) -> Str` — format struct to HTTP wire string
- `http.build_response(status: Int, body: Str) -> Str` — convenience with CORS headers

## Protocol helpers

- `http.status_text(code: Int) -> Str` — reason phrase for a status code
- `http.default_port(scheme: Str) -> Int` — 443 for https/wss, otherwise 80
- `http.build_request_line(method: Str, path: Str) -> Str` — canonical request line

## Legacy (still supported)

- `http.request(method: Str, url: Str, body: Str, timeout_ms: Int) -> Str` — returns body only
- `http.request_status(method: Str, url: Str, body: Str, timeout_ms: Int) -> Int` — status only

Prefer `http.send` for new code.

## Semantics

- `status_text` maps common status codes (`200`, `201`, `204`, `400`–`405`, `422`, `500`)
  and returns `"Unknown"` otherwise.
- Transport: `http://` uses native TCP, `https://` uses system `curl`.

## Benchmark-only helpers

The HTTP server microbenchmark entrypoint is exposed under `bench.http_server_bench` (see
`stdlib/bench/README.md`) and is only available when Vibe is built with `--features bench-runtime`.

## Scope

This preview surface is sync-first and optimized for deterministic service-to-service flows
(timeouts + redirects + explicit status retrieval). Advanced streaming/client pooling will be
tracked in later phases.
