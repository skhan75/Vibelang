#!/usr/bin/env python3
import argparse
import os
import subprocess
import sys
from pathlib import Path


SUITE_COMMANDS = {
    "frontend": [
        ["cargo", "test", "-p", "vibe_cli", "--test", "frontend_fixtures"],
    ],
    "phase12": [
        ["cargo", "test", "-p", "vibe_cli", "--test", "phase12_stdlib"],
        ["cargo", "test", "-p", "vibe_cli", "--test", "phase12_package_ecosystem"],
        ["cargo", "test", "-p", "vibe_cli", "--test", "phase12_test_ergonomics"],
    ],
}


def run_command(repo_root: Path, command: list[str]) -> int:
    print(f"[golden] running: {' '.join(command)}")
    env = dict(os.environ)
    env["UPDATE_GOLDEN"] = "1"
    proc = subprocess.run(command, cwd=repo_root, env=env, check=False)
    return proc.returncode


def main() -> None:
    parser = argparse.ArgumentParser(description="Update deterministic golden snapshots")
    parser.add_argument(
        "--suite",
        choices=["frontend", "phase12", "all"],
        default="frontend",
        help="golden suite to update",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    commands = []
    if args.suite == "all":
        for suite in ("frontend", "phase12"):
            commands.extend(SUITE_COMMANDS[suite])
    else:
        commands.extend(SUITE_COMMANDS[args.suite])

    for command in commands:
        code = run_command(repo_root, command)
        if code != 0:
            print(f"[golden] command failed with exit code {code}")
            sys.exit(code)
    print("[golden] update complete")


if __name__ == "__main__":
    main()
