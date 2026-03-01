# `bench` module (benchmark-only)

This namespace exists **only** for strict third-party benchmark parity (PLB-CI adapters).

It is **not** part of the default VibeLang stdlib surface and is only available when Vibe is built
with the Cargo feature `bench-runtime`.

## APIs

### Hashing / JSON helpers

- `bench.md5_hex(text: Str) -> Str`
- `bench.json_canonical(raw: Str) -> Str`
- `bench.json_repeat_array(item: Str, n: Int) -> Str`

### Benchmark entrypoints

- `bench.http_server_bench(n: Int) -> Int` (requires `@effect net`)
- `bench.secp256k1(n: Int) -> Str`
- `bench.edigits(n: Int) -> Str`

### Socket primitives

These are low-level TCP socket helpers used by benchmark workloads and are not stable APIs:

- `bench.net_listen(host: Str, port: Int) -> Int`
- `bench.net_listener_port(listener_fd: Int) -> Int`
- `bench.net_accept(listener_fd: Int) -> Int`
- `bench.net_connect(host: Str, port: Int) -> Int`
- `bench.net_read(fd: Int, max_bytes: Int) -> Str`
- `bench.net_write(fd: Int, data: Str) -> Int`
- `bench.net_close(fd: Int) -> Bool`

