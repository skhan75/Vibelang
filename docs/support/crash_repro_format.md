# Deterministic Crash Repro Artifact Format

Date: 2026-02-22

## Purpose

Standardize bug-report artifacts so failures are reproducible across machines.

## Artifact Format

Primary metadata file: `crash_repro_sample.json`

Required fields:

- `format`: `vibe-crash-repro-v1`
- `toolchain`: rustc and active toolchain summary
- `command`: full command used to trigger failure
- `exit_code`: process exit code
- `stdout` / `stderr`: captured output
- `binary_sha256`: produced binary checksum
- `source_sha256`: source fixture checksum
- `debug_map`: artifact file name
- `unsafe_audit`: artifact file name
- `alloc_profile`: artifact file name

Optional fields:

- `core_dump_path` (if available)
- `stack_context` (symbolized frames when available)

## Determinism Rules

- Field names and ordering are stable.
- Artifact references use relative names in report payload.
- Timestamps are excluded unless explicitly required.

## Collector Tool

Use:

```bash
python3 tooling/debugging/collect_crash_repro.py
```

Output:

- `reports/phase13/crash_repro_sample.json`
- `reports/phase13/crash_repro_sample.md`
