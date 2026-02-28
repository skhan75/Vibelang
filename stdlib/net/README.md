# `net` module (preview)

Low-level TCP socket primitives.

## APIs

- `net.listen(host: Str, port: Int) -> Int`
- `net.listener_port(listener_fd: Int) -> Int`
- `net.accept(listener_fd: Int) -> Int`
- `net.connect(host: Str, port: Int) -> Int`
- `net.read(fd: Int, max_bytes: Int) -> Str`
- `net.write(fd: Int, data: Str) -> Int`
- `net.close(fd: Int) -> Bool`

## Notes

- These APIs currently use OS sockets and are primarily intended for benchmark parity work.

