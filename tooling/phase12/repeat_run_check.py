#!/usr/bin/env python3
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def run(repo_root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )


def fail(message: str) -> None:
    print(f"phase12 repeat-run check failed: {message}")
    sys.exit(1)


def normalize_test_json(raw: str) -> dict:
    start = raw.find("{")
    end = raw.rfind("}")
    if start == -1 or end == -1 or end <= start:
        fail(f"unable to parse JSON summary from vibe test output:\n{raw}")
    data = json.loads(raw[start : end + 1])
    data.pop("duration_ms", None)
    return data


def check_stdlib_determinism(repo_root: Path, temp_root: Path) -> None:
    source = temp_root / "deterministic_stdlib.yb"
    source.write_text(
        """
pub main() -> Int {
  @effect io
  println(path.join("/a", "b"))
  println(json.minify("{ \\"x\\" : 1 }"))
  println(http.build_request_line("GET", "/ready"))
  println(json.stringify_i64(time.duration_ms(2)))
  0
}
""".strip()
        + "\n"
    )
    first = run(repo_root, ["cargo", "run", "-q", "-p", "vibe_cli", "--", "run", str(source)])
    second = run(repo_root, ["cargo", "run", "-q", "-p", "vibe_cli", "--", "run", str(source)])
    if first.returncode != 0 or second.returncode != 0:
        fail(
            "stdlib determinism run failed:\n"
            f"first rc={first.returncode}\nfirst stderr={first.stderr}\n"
            f"second rc={second.returncode}\nsecond stderr={second.stderr}"
        )
    if first.stdout != second.stdout:
        fail("stdlib output changed between repeated runs")


def check_lockfile_determinism(repo_root: Path, temp_root: Path) -> None:
    project = temp_root / "pkg_project"
    mirror = temp_root / "mirror"
    (project).mkdir(parents=True, exist_ok=True)
    (mirror / "math" / "1.0.0").mkdir(parents=True, exist_ok=True)
    (project / "vibe.toml").write_text(
        "[package]\nname = \"repeat\"\nversion = \"0.1.0\"\n\n[dependencies]\nmath = \"^1.0.0\"\n"
    )
    (mirror / "math" / "1.0.0" / "vibe.toml").write_text(
        "[package]\nname = \"math\"\nversion = \"1.0.0\"\nlicense = \"MIT\"\n"
    )
    first = run(
        repo_root,
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "pkg",
            "lock",
            "--path",
            str(project),
            "--mirror",
            str(mirror),
        ],
    )
    if first.returncode != 0:
        fail("pkg lock first run failed")
    lock_text_1 = (project / "vibe.lock").read_text()
    second = run(
        repo_root,
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "pkg",
            "lock",
            "--path",
            str(project),
            "--mirror",
            str(mirror),
        ],
    )
    if second.returncode != 0:
        fail("pkg lock second run failed")
    lock_text_2 = (project / "vibe.lock").read_text()
    if lock_text_1 != lock_text_2:
        fail("lockfile content changed between repeated runs")


def check_test_json_determinism(repo_root: Path, temp_root: Path) -> None:
    suite = temp_root / "suite"
    suite.mkdir(parents=True, exist_ok=True)
    (suite / "a.yb").write_text("pub main() -> Int { 0 }\n")
    (suite / "b.yb").write_text("pub main() -> Int { 0 }\n")
    first = run(
        repo_root,
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "test",
            str(suite),
            "--json",
        ],
    )
    second = run(
        repo_root,
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "test",
            str(suite),
            "--json",
        ],
    )
    if first.returncode != 0 or second.returncode != 0:
        fail("vibe test json determinism command failed")
    first_json = normalize_test_json(first.stdout)
    second_json = normalize_test_json(second.stdout)
    if first_json != second_json:
        fail("vibe test JSON summary changed between repeated runs")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    with tempfile.TemporaryDirectory(prefix="vibe_phase12_repeat_") as raw_tmp:
        temp_root = Path(raw_tmp)
        check_stdlib_determinism(repo_root, temp_root)
        check_lockfile_determinism(repo_root, temp_root)
        check_test_json_determinism(repo_root, temp_root)
    print("phase12 repeat-run determinism check passed")


if __name__ == "__main__":
    main()
