#!/usr/bin/env python3
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


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    report_dir = repo_root / "reports" / "phase13"
    report_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="vibe_debug_workflow_") as temp_dir:
        source = Path(temp_dir) / "debug_workflow.yb"
        source.write_text(
            """pub main() -> Int {
  @effect io
  @effect alloc
  @effect mut_state
  // @unsafe begin: runtime syscall shim for debug workflow verification
  // @unsafe review: SEC-2026-DBG-0001
  // @unsafe end
  items := []
  items.append(1)
  println("debug workflow smoke")
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
                "debug workflow smoke build failed\n"
                f"stdout:\n{build.stdout}\n"
                f"stderr:\n{build.stderr}"
            )

        run_out = run(
            ["cargo", "run", "-q", "-p", "vibe_cli", "--", "run", str(source)], repo_root
        )
        if run_out.returncode != 0:
            raise SystemExit(
                "debug workflow smoke run failed\n"
                f"stdout:\n{run_out.stdout}\n"
                f"stderr:\n{run_out.stderr}"
            )

        artifact_root = (
            source.parent / ".yb" / "artifacts" / "dev" / "x86_64-unknown-linux-gnu"
        )
        stem = "debug_workflow"
        debug_map = artifact_root / f"{stem}.debug.map"
        unsafe_audit = artifact_root / f"{stem}.unsafe.audit.json"
        alloc_profile = artifact_root / f"{stem}.alloc.profile.json"
        for path in (debug_map, unsafe_audit, alloc_profile):
            if not path.exists():
                raise SystemExit(f"missing debug workflow artifact: {path}")

        report = {
            "format": "phase13-debug-workflow-v1",
            "source_fixture": "debug_workflow.yb",
            "debuginfo_mode": "full",
            "runtime_output": run_out.stdout.strip(),
            "artifact_presence": {
                "debug_map": debug_map.exists(),
                "unsafe_audit": unsafe_audit.exists(),
                "alloc_profile": alloc_profile.exists(),
            },
        }
        (report_dir / "debugging_workflow.json").write_text(
            json.dumps(report, indent=2) + "\n"
        )
        (report_dir / "debugging_workflow.md").write_text(
            "# Phase 13 Debugging/Profiling Workflow Report\n\n"
            "- status: pass\n"
            "- debuginfo_mode: full\n"
            f"- runtime_output: `{report['runtime_output']}`\n"
            "- artifacts:\n"
            "  - debug_map: present\n"
            "  - unsafe_audit: present\n"
            "  - alloc_profile: present\n"
        )
        print(f"wrote {report_dir / 'debugging_workflow.json'}")


if __name__ == "__main__":
    main()
