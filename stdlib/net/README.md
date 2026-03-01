# `net` module (preview)

## APIs

- `net.listen(host: Str, port: Int) -> Int`
- `net.listener_port(listener_fd: Int) -> Int`
- `net.accept(listener_fd: Int) -> Int`
- `net.connect(host: Str, port: Int) -> Int`
- `net.read(fd: Int, max_bytes: Int) -> Str`
- `net.write(fd: Int, data: Str) -> Int`
- `net.close(fd: Int) -> Bool`
- `net.resolve(host: Str) -> Str`

## Semantics

- `listen` binds and listens on IPv4 socket; `port=0` requests an ephemeral port.
- `listener_port` returns bound port for an open listener.
- `connect/read/write/close` are deterministic wrappers over TCP socket primitives.
- `resolve` returns the first resolved IPv4 address string for host, or `""` when resolution fails.

## Error model

- APIs use return-value contracts instead of panicking on expected network failures:
  - `listen/connect/accept/write` return `0` on failure.
  - `read` returns `""` on failure/EOF.
  - `close` returns `false` on failure.
  - `resolve` returns `""` when no address can be resolved.

## TLS plan (current milestone)

- Direct `net.tls_*` socket wrappers are deferred in this preview tier.
- HTTPS/TLS transport is provided by the sync HTTP client surface (`std.http`) so application
  boundaries can use TLS in this milestone while lower-level TLS socket APIs are finalized.

