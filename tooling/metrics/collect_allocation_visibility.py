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
    reports_root = repo_root / "reports" / "v1"
    reports_root.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="vibe_alloc_visibility_") as temp_dir:
        source = Path(temp_dir) / "alloc_visibility.yb"
        source.write_text(
            """pub main() -> Int {
  @effect alloc
  data := []
  data.append(1)
  data.len()
}
"""
        )
        build = run(
            ["cargo", "run", "-q", "-p", "vibe_cli", "--", "build", str(source)],
            repo_root,
        )
        if build.returncode != 0:
            raise SystemExit(
                "allocation visibility smoke failed during build\n"
                f"stdout:\n{build.stdout}\n"
                f"stderr:\n{build.stderr}"
            )

        profile_path = (
            source.parent
            / ".yb"
            / "artifacts"
            / "dev"
            / "x86_64-unknown-linux-gnu"
            / "alloc_visibility.alloc.profile.json"
        )
        if not profile_path.exists():
            raise SystemExit(f"missing allocation profile artifact: {profile_path}")

        profile = json.loads(profile_path.read_text())
        summary = profile.get("summary", {})
        alloc_observed_count = int(summary.get("alloc_observed_count", 0))
        if alloc_observed_count <= 0:
            raise SystemExit(
                "allocation visibility smoke expected alloc_observed_count > 0 "
                f"but got {alloc_observed_count}"
            )

        report = {
            "format": "v1-allocation-visibility-smoke-v1",
            "source_fixture": "alloc_visibility.yb",
            "alloc_observed_count": alloc_observed_count,
            "profile_artifact": "alloc_visibility.alloc.profile.json",
        }
        (reports_root / "allocation_visibility_smoke.json").write_text(
            json.dumps(report, indent=2) + "\n"
        )
        (reports_root / "allocation_visibility_smoke.md").write_text(
            "# Allocation Visibility Smoke\n\n"
            "- status: pass\n"
            "- source_fixture: `alloc_visibility.yb`\n"
            f"- alloc_observed_count: {alloc_observed_count}\n"
            "- profile_artifact: `alloc_visibility.alloc.profile.json`\n"
        )
        print(f"wrote {reports_root / 'allocation_visibility_smoke.json'}")


if __name__ == "__main__":
    main()
