#!/usr/bin/env python3
import json
import re
import subprocess
import tempfile
from pathlib import Path

from jsonrpc_client import JsonRpcClient


def fail(message: str) -> None:
    print(f"phase13 diagnostics parity failed: {message}")
    raise SystemExit(1)


def run(cmd: list[str], cwd: Path) -> tuple[int, str, str]:
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout, completed.stderr


def extract_codes(text: str) -> set[str]:
    return set(re.findall(r"E\d{4}", text))


def file_uri(path: Path) -> str:
    return f"file://{path.as_posix()}"


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    fixture = (
        repo_root
        / "compiler"
        / "tests"
        / "fixtures"
        / "phase7"
        / "basic"
        / "typecheck"
        / "typecheck__unknown_symbol_and_mismatch.yb"
    )
    if not fixture.exists():
        fail(f"parity fixture missing: {fixture}")
    source = fixture.read_text()

    with tempfile.TemporaryDirectory(prefix="vibe_phase13_parity_") as temp_dir:
        temp_fixture = Path(temp_dir) / fixture.name
        temp_fixture.write_text(source)

        status, stdout, stderr = run(
            ["cargo", "run", "-q", "-p", "vibe_cli", "--", "check", str(temp_fixture)],
            repo_root,
        )
        if status not in (0, 1):
            fail(f"`vibe check` failed unexpectedly:\nstdout:\n{stdout}\nstderr:\n{stderr}")
        cli_codes = extract_codes(stdout + "\n" + stderr)
        if not cli_codes:
            fail("no diagnostic codes captured from `vibe check` output")

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
        client.request("initialize", {})
        client.notify(
            "textDocument/didOpen",
            {
                "textDocument": {
                    "uri": file_uri(temp_fixture),
                    "version": 1,
                    "languageId": "vibelang",
                    "text": source,
                }
            },
        )
        publish = client.wait_for_notification("textDocument/publishDiagnostics")
        diagnostics = publish.get("params", {}).get("diagnostics", [])
        lsp_codes = {
            code
            for code in (
                d.get("code")
                for d in diagnostics
                if isinstance(d, dict) and isinstance(d.get("code"), str)
            )
            if code
        }
        client.request("shutdown", {})
        client.notify("exit", {})
        client.close()

    if cli_codes != lsp_codes:
        fail(
            "diagnostic code parity mismatch: "
            f"cli={sorted(cli_codes)} lsp={sorted(lsp_codes)}"
        )

    reports_root = repo_root / "reports" / "phase13"
    reports_root.mkdir(parents=True, exist_ok=True)
    report_json = reports_root / "editor_ci_consistency.json"
    report_json.write_text(
        json.dumps(
            {
                "fixture": str(fixture),
                "cli_codes": sorted(cli_codes),
                "lsp_codes": sorted(lsp_codes),
                "parity": True,
            },
            indent=2,
        )
        + "\n"
    )
    print(f"phase13 diagnostics parity passed; wrote {report_json}")


if __name__ == "__main__":
    main()

