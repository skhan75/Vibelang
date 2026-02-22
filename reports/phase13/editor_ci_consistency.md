# Phase 13.1 Editor/CI Consistency Report

Date: 2026-02-22

## Goal

Verify that diagnostics surfaced in editor LSP flow match CLI diagnostics for the
same source fixture and that consistency checks are CI-gated.

## Parity execution

Command:

```bash
python3 tooling/phase13/check_diagnostics_parity.py
```

Output artifact:

- `reports/phase13/editor_ci_consistency.json`

## Result

- Fixture: `compiler/tests/fixtures/phase7/basic/typecheck/typecheck__unknown_symbol_and_mismatch.yb`
- CLI diagnostic codes: `E2001`, `E2102`
- LSP diagnostic codes: `E2001`, `E2102`
- Parity: `true`

## CI integration

Consistency gate is included in:

- `.github/workflows/phase13-editor-ux.yml`

The workflow also enforces protocol smoke and benchmark budgets to keep editor
behavior aligned with CI expectations.

