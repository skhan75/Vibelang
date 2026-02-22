#!/usr/bin/env python3
import json
import tempfile
from pathlib import Path

from jsonrpc_client import JsonRpcClient


def fail(message: str) -> None:
    print(f"phase13 protocol smoke failed: {message}")
    raise SystemExit(1)


def file_uri(path: Path) -> str:
    return f"file://{path.as_posix()}"


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "compiler" / "tests" / "fixtures" / "phase7" / "basic" / "syntax" / "syntax__literals_and_comments.yb"
    if not fixture.exists():
        fail(f"fixture missing: {fixture}")
    source = fixture.read_text()

    with tempfile.TemporaryDirectory(prefix="vibe_phase13_protocol_") as temp_dir:
        index_root = Path(temp_dir) / "index"
        command = [
            "cargo",
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "lsp",
            "--transport",
            "jsonrpc",
            "--index-root",
            str(index_root),
        ]
        client = JsonRpcClient(command, cwd=str(repo_root))

        initialize, initialize_ms = client.request("initialize", {})
        if "result" not in initialize:
            fail("initialize response missing result")

        uri = file_uri(fixture)
        client.notify(
            "textDocument/didOpen",
            {
                "textDocument": {
                    "uri": uri,
                    "version": 1,
                    "languageId": "vibelang",
                    "text": source,
                }
            },
        )
        published = client.wait_for_notification("textDocument/publishDiagnostics")
        if published.get("params", {}).get("uri") != uri:
            fail("didOpen diagnostics notification uri mismatch")

        completion, completion_ms = client.request(
            "textDocument/completion",
            {
                "textDocument": {"uri": uri},
                "position": {"line": 0, "character": 3},
            },
        )
        items = completion.get("result", {}).get("items", [])
        if not isinstance(items, list):
            fail("completion response missing items array")

        rename, rename_ms = client.request(
            "textDocument/rename",
            {
                "textDocument": {"uri": uri},
                "position": {"line": 0, "character": 1},
                "newName": "mainRenamed",
            },
        )
        if "result" not in rename:
            fail("rename response missing result")

        formatting, formatting_ms = client.request(
            "textDocument/formatting",
            {"textDocument": {"uri": uri}, "options": {"tabSize": 2, "insertSpaces": True}},
        )
        if not isinstance(formatting.get("result"), list):
            fail("formatting result must be text edit array")

        shutdown, shutdown_ms = client.request("shutdown", {})
        if shutdown.get("result") is not None:
            fail("shutdown should return null result")
        client.notify("exit", {})
        return_code, _, stderr = client.close()
        if return_code not in (0, None):
            fail(f"lsp process exited with code {return_code}: {stderr}")

    summary = {
        "initialize_ms": initialize_ms,
        "completion_ms": completion_ms,
        "rename_ms": rename_ms,
        "formatting_ms": formatting_ms,
        "shutdown_ms": shutdown_ms,
    }
    print("phase13 protocol smoke passed")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()

