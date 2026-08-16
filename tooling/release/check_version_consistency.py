#!/usr/bin/env python3
"""Fail when the version VibeLang advertises disagrees across its sources.

Compared sources:

  1. `version` under `[workspace.package]` in `Cargo.toml`, which is what
     `vibe --version` prints
  2. every `vX.Y.Z` reference in `README.md`, meaning the badge and any
     download or install instruction
  3. the newest released entry in `CHANGELOG.md`, written `## [X.Y.Z]`, with
     `## [Unreleased]` skipped
  4. the tag being cut, when one is supplied

Comparison rules:

  - Cargo.toml, CHANGELOG.md and the tag must match exactly, pre-release
    identifier included. Cutting `v1.7.0` from a tree that still says
    `1.7.0-rc.1` is a failure.
  - README references are compared on `major.minor.patch` only, so the README
    may advertise `v1.7.0` while a release candidate is in the tree. This also
    keeps the trailing colour in a shields.io badge URL from being read as a
    pre-release identifier.

The tag comes from `--tag`, or from `GITHUB_REF_NAME` when the workflow was
triggered by a tag push. Without a tag the first three sources are still
compared, so a disagreement is caught on the pull request that introduces it
rather than at tag time.

Exit codes: 0 agreement, 1 disagreement, 2 a source could not be read.
"""

import argparse
import os
import re
import sys
from pathlib import Path

SEMVER = r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?"
CARGO_SECTION_RE = re.compile(r"^\[(?P<name>[^\]]+)\]\s*$")
CARGO_VERSION_RE = re.compile(r'^\s*version\s*=\s*"(?P<version>' + SEMVER + r')"\s*$')
README_VERSION_RE = re.compile(
    r"(?<![0-9A-Za-z.])v(?P<version>\d+\.\d+\.\d+)(?![0-9A-Za-z])(?!\.\d)"
)
CHANGELOG_HEADING_RE = re.compile(r"^##\s*\[(?P<label>[^\]]+)\]")


class SourceError(Exception):
    """A file that must be compared could not be read or parsed."""


def core(version: str) -> str:
    """Return `major.minor.patch`, dropping any pre-release identifier."""
    return version.split("-", 1)[0]


def read_lines(path: Path) -> list[str]:
    if not path.exists():
        raise SourceError(f"missing file: {path}")
    return path.read_text(encoding="utf-8").splitlines()


def cargo_workspace_version(path: Path) -> tuple[str, str]:
    """Return the `[workspace.package]` version and where it was found."""
    section = ""
    for number, line in enumerate(read_lines(path), start=1):
        section_match = CARGO_SECTION_RE.match(line)
        if section_match:
            section = section_match.group("name").strip()
            continue
        if section != "workspace.package":
            continue
        version_match = CARGO_VERSION_RE.match(line)
        if version_match:
            return version_match.group("version"), f"{path.name}:{number}"
    raise SourceError(f"no `version` under `[workspace.package]` in {path}")


def readme_versions(path: Path) -> tuple[set[str], list[str]]:
    """Return every `vX.Y.Z` referenced in the README and where each was found."""
    versions: set[str] = set()
    locations: list[str] = []
    for number, line in enumerate(read_lines(path), start=1):
        for match in README_VERSION_RE.finditer(line):
            versions.add(match.group("version"))
            locations.append(f"{path.name}:{number}")
    if not versions:
        raise SourceError(
            f"no `vX.Y.Z` reference found in {path}; the README must state the "
            "released version so this check has something to compare"
        )
    return versions, locations


def changelog_newest_release(path: Path) -> tuple[str, str]:
    """Return the newest released version heading, skipping `Unreleased`."""
    for number, line in enumerate(read_lines(path), start=1):
        heading = CHANGELOG_HEADING_RE.match(line)
        if not heading:
            continue
        label = heading.group("label").strip()
        if label.lower() == "unreleased":
            continue
        if not re.fullmatch(SEMVER, label):
            raise SourceError(
                f"{path.name}:{number}: release heading `[{label}]` is not a "
                "semantic version"
            )
        return label, f"{path.name}:{number}"
    raise SourceError(f"no released `## [X.Y.Z]` entry in {path}")


def tag_version(explicit: str | None) -> tuple[str | None, str]:
    """Return the version implied by the tag being cut, if there is one."""
    tag = explicit
    origin = "--tag"
    if tag is None and os.environ.get("GITHUB_REF_TYPE") == "tag":
        tag = os.environ.get("GITHUB_REF_NAME")
        origin = "GITHUB_REF_NAME"
    if not tag:
        return None, "not supplied"
    stripped = tag[1:] if tag.startswith("v") else tag
    if not re.fullmatch(SEMVER, stripped):
        raise SourceError(f"tag `{tag}` is not of the form vX.Y.Z")
    return stripped, f"{origin}={tag}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--tag",
        help="tag being cut, for example v1.7.0; defaults to the tag that triggered CI",
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
        help="repository root to check (defaults to this script's repository)",
    )
    args = parser.parse_args()

    root: Path = args.repo_root
    try:
        cargo, cargo_at = cargo_workspace_version(root / "Cargo.toml")
        readme, readme_at = readme_versions(root / "README.md")
        changelog, changelog_at = changelog_newest_release(root / "CHANGELOG.md")
        tag, tag_at = tag_version(args.tag)
    except SourceError as error:
        print(f"version consistency check could not run: {error}", file=sys.stderr)
        return 2

    rows = [
        ("Cargo.toml [workspace.package]", cargo, cargo_at),
        ("README.md references", ", ".join(sorted(readme)), ", ".join(sorted(set(readme_at)))),
        ("CHANGELOG.md newest release", changelog, changelog_at),
        ("tag being cut", tag or "none", tag_at),
    ]
    width = max(len(label) for label, _, _ in rows)
    print("version consistency check")
    for label, value, where in rows:
        print(f"  {label.ljust(width)}  {value}  ({where})")

    exact = {cargo, changelog}
    if tag is not None:
        exact.add(tag)
    readme_cores = {core(version) for version in readme}
    failures: list[str] = []
    if len(exact) != 1:
        failures.append(
            "Cargo.toml, CHANGELOG.md and the tag must be the same version, but "
            "they are " + ", ".join(sorted(exact))
        )
    if readme_cores != {core(cargo)}:
        failures.append(
            "README.md advertises "
            + ", ".join(sorted(readme_cores))
            + f" while Cargo.toml says {core(cargo)}"
        )

    if not failures:
        print(f"OK: every source advertises {cargo}")
        return 0

    print("", file=sys.stderr)
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    print(
        "Fix all of these in one commit: `version` under `[workspace.package]` in "
        "Cargo.toml, every `vX.Y.Z` in README.md, the newest `## [X.Y.Z]` entry in "
        "CHANGELOG.md, and the tag. See RELEASING.md.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
