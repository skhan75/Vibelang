#!/usr/bin/env python3
import json
import os
import select
import subprocess
import time


class JsonRpcClient:
    def __init__(self, command: list[str], cwd: str, timeout_s: float = 8.0) -> None:
        self.process = subprocess.Popen(
            command,
            cwd=cwd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=False,
        )
        if self.process.stdin is None or self.process.stdout is None:
            raise RuntimeError("failed to open stdio pipes for jsonrpc client")
        self._stdin = self.process.stdin
        self._stdout = self.process.stdout
        self._default_timeout_s = timeout_s
        self._next_id = 1
        self._queued_messages: list[dict] = []

    def request(self, method: str, params: dict | None = None) -> tuple[dict, int]:
        req_id = self._next_id
        self._next_id += 1
        payload = {
            "jsonrpc": "2.0",
            "id": req_id,
            "method": method,
            "params": params or {},
        }
        start = time.perf_counter()
        self._send(payload)
        response = self._wait_for(lambda msg: msg.get("id") == req_id)
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        if "error" in response:
            raise RuntimeError(f"jsonrpc {method} error: {response['error']}")
        return response, elapsed_ms

    def notify(self, method: str, params: dict | None = None) -> None:
        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or {},
        }
        self._send(payload)

    def wait_for_notification(self, method: str, timeout_s: float | None = None) -> dict:
        timeout = timeout_s if timeout_s is not None else self._default_timeout_s
        msg = self._wait_for(lambda item: item.get("method") == method, timeout)
        return msg

    def close(self) -> tuple[int | None, str, str]:
        try:
            if self.process.stdin and not self.process.stdin.closed:
                self.process.stdin.close()
            self.process.stdin = None
        except Exception:
            pass
        try:
            stdout, stderr = self.process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            stdout, stderr = self.process.communicate()
        return self.process.returncode, (stdout or b"").decode(errors="ignore"), (
            stderr or b""
        ).decode(errors="ignore")

    def _send(self, payload: dict) -> None:
        serialized = json.dumps(payload, separators=(",", ":")).encode()
        framed = b"Content-Length: " + str(len(serialized)).encode() + b"\r\n\r\n" + serialized
        self._stdin.write(framed)
        self._stdin.flush()

    def _wait_for(self, predicate, timeout_s: float | None = None) -> dict:
        timeout = timeout_s if timeout_s is not None else self._default_timeout_s
        deadline = time.time() + timeout

        for idx, queued in enumerate(self._queued_messages):
            if predicate(queued):
                return self._queued_messages.pop(idx)

        while True:
            remaining = deadline - time.time()
            if remaining <= 0:
                raise TimeoutError("timed out waiting for jsonrpc message")
            message = self._read_message(remaining)
            if predicate(message):
                return message
            self._queued_messages.append(message)

    def _read_message(self, timeout_s: float) -> dict:
        content_length = None
        while True:
            header_line = self._read_line(timeout_s)
            if header_line in (b"\r\n", b"\n"):
                break
            if header_line.lower().startswith(b"content-length:"):
                value = header_line.split(b":", 1)[1].strip()
                content_length = int(value.decode())
        if content_length is None:
            raise RuntimeError("jsonrpc message missing Content-Length")
        body = self._read_exact(content_length, timeout_s)
        return json.loads(body.decode())

    def _read_line(self, timeout_s: float) -> bytes:
        out = bytearray()
        while True:
            out.extend(self._read_exact(1, timeout_s))
            if out.endswith(b"\n"):
                return bytes(out)

    def _read_exact(self, size: int, timeout_s: float) -> bytes:
        out = bytearray()
        deadline = time.time() + timeout_s
        while len(out) < size:
            remaining = deadline - time.time()
            if remaining <= 0:
                raise TimeoutError("timed out while reading jsonrpc stream")
            ready, _, _ = select.select([self._stdout], [], [], remaining)
            if not ready:
                raise TimeoutError("timed out waiting for jsonrpc stream readiness")
            chunk = os.read(self._stdout.fileno(), size - len(out))
            if not chunk:
                raise RuntimeError("unexpected EOF while reading jsonrpc stream")
            out.extend(chunk)
        return bytes(out)

