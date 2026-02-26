# VibeLang AI Contributor Guide

This file is the root-level master prompt for AI contributors working in this repository.

## Primary goal

Keep the repository clean, non-redundant, and maintainable. Prefer updating existing artifacts over creating new copies.

## File creation rules

- Do not create duplicate "pointer copies" of the same report/content.
- Do not create timestamped report/docs files unless explicitly requested.
- Prefer one canonical path per artifact type.
- Before creating a new file, check whether an existing file can be updated.
- Avoid creating scratch/debug docs in the repo; keep temporary notes outside versioned paths.

## Documentation rules

- Keep docs concise and actionable.
- Avoid repeating the same guidance across multiple files.
- If a doc supersedes another, update references and remove the old one.
- Do not leave stale links to removed or renamed files.

## Build and generated artifacts

- Never commit generated build/cache outputs (`target/`, `node_modules/`, `.yb/artifacts/`, `.vibe/artifacts/`, etc.) unless explicitly required.
- If generated files appear during work, clean them up before finishing.

## Benchmark/report canonical paths

- Use `reports/benchmarks/third_party/full/results.json` as canonical benchmark JSON.
- Use `reports/benchmarks/third_party/full/summary.md` as canonical human summary.
- Keep history only when it adds real value; avoid duplicate snapshots with identical content.

## Before finalizing any change

- Remove unnecessary files created during the task.
- Verify no references point to deleted/obsolete paths.
- Ensure new files are essential, non-duplicative, and clearly named.
