#!/usr/bin/env python3
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )


def sha256_file(path: Path) -> str:
    data = path.read_bytes()
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    report_dir = repo_root / "reports" / "phase13"
    report_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="vibe_crash_repro_") as temp_dir:
        source = Path(temp_dir) / "crash_probe.yb"
        source.write_text(
            """pub main() -> Int {
  @effect io
  @require 1 == 2
  println("should not reach")
  0
}
"""
        )

        build = run(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "vibe_cli",
                "--",
                "build",
                str(source),
                "--debuginfo",
                "full",
            ],
            repo_root,
        )
        if build.returncode != 0:
            raise SystemExit(
                "crash repro collection failed during build\n"
                f"stdout:\n{build.stdout}\n"
                f"stderr:\n{build.stderr}"
            )

        run_out = run(
            ["cargo", "run", "-q", "-p", "vibe_cli", "--", "run", str(source)], repo_root
        )
        if run_out.returncode == 0:
            raise SystemExit("crash repro probe expected failure but command succeeded")

        artifact_root = (
            source.parent / ".yb" / "artifacts" / "dev" / "x86_64-unknown-linux-gnu"
        )
        stem = "crash_probe"
        binary_path = artifact_root / stem
        debug_map = artifact_root / f"{stem}.debug.map"
        unsafe_audit = artifact_root / f"{stem}.unsafe.audit.json"
        alloc_profile = artifact_root / f"{stem}.alloc.profile.json"
        for path in (binary_path, debug_map, unsafe_audit, alloc_profile):
            if not path.exists():
                raise SystemExit(f"missing required crash repro artifact: {path}")

        rustc_version = run(["rustc", "-Vv"], repo_root)
        active_toolchain = run(["rustup", "show", "active-toolchain"], repo_root)
        source_sha = hashlib.sha256(source.read_bytes()).hexdigest()
        binary_sha = sha256_file(binary_path)
        report = {
            "format": "vibe-crash-repro-v1",
            "toolchain": {
                "rustc_vv": rustc_version.stdout.strip(),
                "active_toolchain": active_toolchain.stdout.strip(),
            },
            "command": f"vibe run {source.name}",
            "exit_code": run_out.returncode,
            "stdout": run_out.stdout.strip(),
            "stderr": run_out.stderr.strip(),
            "binary_sha256": binary_sha,
            "source_sha256": source_sha,
            "debug_map": debug_map.name,
            "unsafe_audit": unsafe_audit.name,
            "alloc_profile": alloc_profile.name,
        }
        (report_dir / "crash_repro_sample.json").write_text(json.dumps(report, indent=2) + "\n")
        (report_dir / "crash_repro_sample.md").write_text(
            "# Crash Repro Sample\n\n"
            "- format: `vibe-crash-repro-v1`\n"
            f"- command: `{report['command']}`\n"
            f"- exit_code: {report['exit_code']}\n"
            f"- binary_sha256: `{report['binary_sha256']}`\n"
            f"- source_sha256: `{report['source_sha256']}`\n"
            f"- debug_map: `{report['debug_map']}`\n"
            f"- unsafe_audit: `{report['unsafe_audit']}`\n"
            f"- alloc_profile: `{report['alloc_profile']}`\n"
        )
        print(f"wrote {report_dir / 'crash_repro_sample.json'}")


if __name__ == "__main__":
    main()
