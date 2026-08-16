# Releasing VibeLang

This is the procedure for cutting a VibeLang release. It is the only document
that lists the steps. The standards behind the steps live in the policy
documents linked from each one, and those documents do not restate the
procedure.

Every step below names the command that proves it and who runs that command. If
a step cannot be proved by a command, it is written as a judgment call and
assigned to the release owner on purpose.

## Why this document exists

The releases published before this document did not follow one procedure, and
the evidence is still in the repository. Each row was verified on 2026-08-16
against the `skhan75/vibelang` remote.

| What happened | How to see it |
| --- | --- |
| The README badge and download command pointed at v1.1.0 while `Cargo.toml` said 1.6.0 | fixed in the commit that added this file |
| `CHANGELOG.md` stopped at 1.0.2, so 1.1.0, 1.1.1, 1.2.0 and 1.6.0 had no entries | backfilled in the commit that added this file |
| Tags jump from v1.2.0 to v1.6.0. 1.3.0, 1.4.0 and 1.5.0 were never released and never appeared in `Cargo.toml` | `git tag --sort=creatordate`, `git log -S'version = "1.3.0"' -- Cargo.toml` |
| Every release after v1.0.0 has zero attached assets, so the README download command returned 404 while the v1.0.0 one returned 200. v1.0.2 has a tag and no release at all | `gh release view v1.6.0 --json assets`, `curl -sI -o /dev/null -w '%{http_code}' -L <url>` |
| v1.6.0 was tagged from commit `16cc807`, where 25 of 88 checks had already failed, including `tests`, `fmt_lint` and `packaging_integrity_smoke` | `gh api repos/skhan75/vibelang/commits/16cc807/check-runs?per_page=100` |
| v1.0.0 was tagged with `Cargo.toml` at 0.1.0, and v1.1.1 with `Cargo.toml` at 1.1.0, so the binary reported a version the release did not | `git show v1.0.0:Cargo.toml`, `git show v1.1.1:Cargo.toml` |

`docs/policy/versioning_compatibility.md` already required release notes and a
migration guide when those releases were cut. The policy was not missing. It
was not followed. So the parts of it that a machine can check are now checked by
a machine: `tooling/release/check_version_consistency.py`, run by the
`release-version-consistency` workflow on pushes to `main` and `release/**`, on
pull requests targeting them, and on every `v*` tag.

## Roles

- **Release owner.** One named person per release. Runs the steps, and is the
  only person who creates a tag or publishes a GitHub release.
- **CI.** The workflows named in each step. Wherever a precondition can be
  machine checked, CI is the verifier and the owner's job is to read the result
  rather than to assert it.

## Choosing the version number

The version lives in exactly one place, `version` under `[workspace.package]`
in `Cargo.toml`. Everything else follows it: `vibe --version` reads it through
`CARGO_PKG_VERSION`, and the README, the CHANGELOG and the tag must match it.

Pick the number from what changed, using the rules in
`docs/policy/versioning_compatibility.md`:

- **PATCH** when only fixes ship and every program that compiled before still
  compiles and behaves the same.
- **MINOR** when new language or tooling surface is added and existing programs
  are unaffected.
- **MAJOR** when a program that compiled and behaved correctly under the
  previous release no longer does. Fixing a miscompilation counts, because it
  changes the behavior of code that compiles today.

Two rules that the history above shows were not obvious:

- Do not skip numbers. The next release is the next number on the line. A jump
  from 1.2.0 to 1.6.0 tells a reader that three releases exist somewhere, and
  they do not.
- Release candidates use a pre-release identifier, `1.7.0-rc.1` and so on, per
  `docs/release/rc_process.md`. Note the cost before using one: `[vibelang]`
  version requirements in `vibe.toml` are matched with the `semver` crate in
  `crates/vibe_pkg/src/lib.rs`, and a pre-release version does not satisfy a
  plain requirement. A project pinning `1.6` rejects a `1.7.0-rc.1` compiler.
  Release candidates are for testers who pin nothing.

## Current version recommendation

**Recommendation: stay on the 1.x line, leave the published tags alone, and pay
the debt the 1.x number implies instead of lowering the number.** Concretely,
keep the source at 1.6.0 today, cut the next release as 1.7.0 if it only adds,
and as 2.0.0 when the soundness fixes change the behavior of programs that
compile now. Alongside that, replace the open-ended sentence in
`docs/policy/versioning_compatibility.md` about "stable language constructs"
with a list of which constructs are actually covered.

The argument, in the order that matters:

1. **Renumbering cannot retract what is published.** Six releases on the 1.x
   line are already published, from v1.0.0 to v1.6.0. A reset to 0.x changes
   what the next release claims, not what those six claimed. The honesty
   problem is retrospective and a new number does not reach it.
2. **A 0.x reset breaks the pin mechanic.** `check_compiler_version` in
   `crates/vibe_pkg/src/lib.rs` matches the compiler version against the
   `[vibelang]` requirement with `semver::VersionReq`. A project pinning
   `>=1.2` rejects a 0.7.0 compiler. How many projects pin this way is unknown,
   so this is a real cost of unmeasured size, not a catastrophe.
3. **The number is not where the promise is broken.** The README says Beta and
   says syntax, stdlib and toolchain APIs may change between releases. The
   compatibility policy says source compatibility is maintained for stable
   language constructs and never says which constructs those are. Two documents
   make opposite claims and a reader can check neither. Going to 0.7.0 does not
   settle that. Enumerating the surface does, and the policy already owes it.
4. **What actually undercuts 1.x today is correctness, and that argues for a
   MAJOR, not a retreat.** `todo.md` records reproduced miscompilations dated
   2026-08-15, including an assignment inside a `match` arm that is silently a
   no-op and an inliner that evaluates a side-effecting argument the wrong
   number of times. Fixing those changes what compiling programs do. Under
   SemVer that is 2.0.0 with a migration note, which is a stronger and more
   specific signal than 0.x.

**The trade-off, stated plainly.** 0.x is the louder public signal, and it is
the better choice if the owner expects most of the next several releases to
break programs rather than one of them. The price is the pins in point 2, a
version line that reads as a regression to anyone who saw 1.6.0, and install
instructions in the wild that stop resolving. If the soundness program is
expected to break the language repeatedly, that price is worth paying. If it is
expected to break it once, 2.0.0 says the same thing without the collateral.

Re-tagging, renumbering, or deleting any published release is the owner's
decision. Writing this document changed no version and created no tag.

## Preconditions

All of these must hold for the exact commit being tagged. Not a similar commit,
and not the branch tip an hour later. Record the commit SHA first and use it in
every command:

```bash
SHA=$(git rev-parse HEAD)
```

| # | Precondition | Command that proves it | Verified by |
| --- | --- | --- | --- |
| 1 | The version agrees everywhere | `python3 tooling/release/check_version_consistency.py --tag vX.Y.Z` | `release-version-consistency` workflow, and the owner locally |
| 2 | Format and lint clean | `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` | `fmt_lint` job in `phase1-frontend.yml` |
| 3 | Tests pass | `cargo test --workspace --all-targets` | `tests` job in `phase1-frontend.yml` |
| 4 | Examples pass | `bash tooling/test_all_examples.sh` | release owner. No workflow runs this today, so it is a manual step until one does |
| 5 | No failing check on the commit | `gh api repos/skhan75/vibelang/commits/$SHA/check-runs?per_page=100 --jq '[.check_runs[] \| select(.conclusion == "failure") \| .name]'` prints `[]` | release owner. An empty list also means no checks ran at all, which is why precondition 6 exists |
| 6 | Release gates completed, not skipped | `gh api repos/skhan75/vibelang/commits/$SHA/check-runs?per_page=100 --jq '.check_runs[] \| select(.name == "summary") \| .conclusion'` prints `success` | `summary` job in `v1-release-gates.yml`, which needs 40 gate jobs. At `16cc807` it printed `skipped` |
| 7 | Release notes validate | `python3 tooling/release/validate_release_notes.py` | `release_notes_automation_gate` in `v1-release-gates.yml` |
| 8 | CHANGELOG entry for this version exists and is filled in | read it | release owner. The gate checks the version number, not whether the prose is true |
| 9 | Migration guide exists for every breaking change | `ls docs/migrations/` | release owner, per `docs/policy/versioning_compatibility.md` |
| 10 | RC soak completed for MAJOR and MINOR releases | `docs/release/rc_process.md`, minimum 24 hours | release owner |

If any of these fails, there is no tag yet. That is the whole rule that was
missing when v1.6.0 was cut.

## Cutting the release

1. **Open a release preparation pull request.** In one commit, set
   `[workspace.package].version` in `Cargo.toml`, every `vX.Y.Z` in
   `README.md`, and a new `## [X.Y.Z] — YYYY-MM-DD` section in `CHANGELOG.md`
   to the chosen version. The `release-version-consistency` workflow fails the
   pull request if these disagree.
2. **Wait for CI on that pull request.** Do not merge on a red run. Do not
   merge and fix afterwards.
3. **Merge, then re-check preconditions on the merge commit.** The merge commit
   is what gets tagged, and it is not the commit CI checked on the pull
   request. Run preconditions 1, 5 and 6 again against `$SHA`.
4. **Tag the verified commit.**
   ```bash
   git tag -a vX.Y.Z -m "VibeLang vX.Y.Z" "$SHA"
   python3 tooling/release/check_version_consistency.py --tag vX.Y.Z
   git push origin vX.Y.Z
   ```
   The push triggers `release-version-consistency` on the tag. If that run is
   red, delete the tag before doing anything else.
5. **Produce the assets.** See the next section. Do not publish the release
   before the assets exist.
6. **Publish the GitHub release** with the notes generated by
   `tooling/release/generate_release_notes.py`, the assets attached, and the
   CHANGELOG section as the body's source of truth.
7. **Verify what a user sees.** Fetch the published assets from a clean
   directory, exactly as the README instructs, and run `vibe --version`. The
   output must equal the tag. Skipping this step is how the README came to
   document a download that does not exist.

## Release assets

Assets are built by `.github/workflows/v1-packaged-release.yml`. For each of
`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin` and `x86_64-pc-windows-msvc`
it builds `vibe` with the pinned toolchain, archives it, writes a SHA-256
checksum file, generates an SPDX SBOM, signs the archive, SBOM and provenance
with cosign, verifies those signatures, and runs an install smoke test on each
platform.

**Known gap, as of 2026-08-16.** That workflow uploads its output with
`actions/upload-artifact`, which puts the files on the workflow run and not on
the GitHub release. No workflow in `.github/workflows/` contains a
`gh release upload` or equivalent step, which is why every release published
after v1.0.0 has zero assets. It also never runs on a tag: its triggers are
pushes to `main` and `release/**`, pull requests, and manual dispatch. Until an
upload step exists, attaching assets is a manual step for the release owner:

```bash
gh run download <run-id> --dir dist/
find dist -type f -exec gh release upload vX.Y.Z {} +
gh release view vX.Y.Z --json assets --jq '[.assets[].name]'
```

The last command is the verification. A release with an empty asset list is not
finished, and the README must not describe a download that does not exist.

Closing this gap by adding an upload step to `v1-packaged-release.yml`, gated
on a tag push, is the single highest value change to this process. It is left
as a decision for the owner because it changes what a tag push does.

## Changelog

`CHANGELOG.md` is written before the tag, never after. The entry for a version
is part of the release preparation pull request, and the version consistency
gate fails if the newest entry does not match `Cargo.toml`.

Each entry carries the version and date as `## [X.Y.Z] — YYYY-MM-DD`, then only
the sections that apply, from `Added`, `Changed`, `Fixed`, and
`Migration Notes` when anything breaks. Write what a user can observe. If a
claim cannot be checked against the tagged tree, leave it out. The backfilled
1.1.0 entry omits four finding codes that the published release notes listed as
shipped, because `git grep I5005 v1.1.0 -- crates/` finds nothing: at that tag
the codes existed in `ai/sidecar/architecture.md` and in no code that could
emit them.

Work merged between releases goes under `## [Unreleased]`, which the gate
ignores.

## Rollback

`docs/release/rollback_playbook.md` is the procedure, including the trigger
signals, the P0 and P1 decision matrix, and the communication template. Two
points specific to the mechanics here:

- **Prefer fixing forward.** A published tag has been fetched by people and by
  caches. Moving or deleting it makes two different trees answer to one name.
  Cut the next PATCH instead, and say in its CHANGELOG entry what it withdraws.
- **Delete only what nobody could have used.** A tag pushed minutes ago whose
  release was never published and whose assets were never attached can be
  deleted with `git push origin :refs/tags/vX.Y.Z`. Once a release is published
  with assets, mark it as a pre-release or add a prominent note to its body
  instead, and leave the artifacts in place so existing checksums keep
  resolving.

After any rollback, re-run preconditions 1 through 7 against the commit that
users are now directed to, and record the outcome in the release notes for the
replacement version.

## Related documents

- `docs/policy/versioning_compatibility.md`: what each version number promises.
- `docs/release/rc_process.md`: RC sequence, soak window, promote and reject
  criteria.
- `docs/release/release_notes_policy.md`: required release note sections and
  the generator.
- `docs/release/rollback_playbook.md`: full rollback procedure.
- `docs/release/release_blocker_policy.md`: P0 and P1 definitions.
- `docs/release/process.md`: the evidence bundle a release candidate links.
