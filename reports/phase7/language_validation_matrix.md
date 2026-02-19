# Phase 7.1 Language Validation Matrix

Date: 2026-02-17

## Coverage Matrix

| Feature Area | Basic | Intermediate | Advanced | Stress | Evidence |
| --- | --- | --- | --- | --- | --- |
| Syntax + Lexing | PASS | - | - | - | `frontend_fixtures.rs` test `phase7_basic_and_intermediate_matrix` |
| Identifiers + Parser Recovery | PASS | - | - | - | phase7 basic fixtures + `.diag` goldens |
| Type Checking Boundaries | PASS | PASS | - | - | phase7 basic/intermediate type fixtures |
| Annotation Validity/Invalidity | - | PASS | - | - | phase7 intermediate annotation fixtures |
| Effect Conformance/Drift | - | PASS | PASS | - | annotation drift fixture + intent drift fixture |
| Single-thread Sample Programs | - | - | PASS | - | `phase7_validation.rs` |
| Concurrency Patterns | - | - | PASS | PASS | `phase7_concurrency.rs` |
| Concurrency Misuse Diagnostics | - | - | PASS | - | phase7 concurrency_err `.diag` fixtures |
| Intent Lint Match/Drift/Changed Mode | - | - | PASS | - | `phase7_intent_validation.rs` |
| Verifier-gated Suggestion Rejection | - | - | PASS | - | `phase7_intent_validation.rs` |
| Intent Lint Quality Trend | - | - | PASS | PASS | `reports/phase7/intent_lint_quality_trend.json` |

## Validation Commands

- `cargo test -p vibe_cli --test frontend_fixtures phase7_basic_and_intermediate_matrix`
- `cargo test -p vibe_cli --test frontend_fixtures phase7_frontend_outputs_are_deterministic`
- `cargo test -p vibe_cli --test phase7_validation`
- `cargo test -p vibe_cli --test phase7_concurrency`
- `cargo test -p vibe_cli --test phase7_intent_validation`
- `python3 tooling/metrics/collect_intent_lint_quality.py`
- `python3 tooling/metrics/validate_intent_lint_quality.py`

## Result

- Current local run status: PASS for all listed commands.
