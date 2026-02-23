# Pilot Case Study: CLI Tool Reference

- pilot: `pilot-apps/cli_tool_reference/main.yb`
- class: CLI/tooling workflow with contracts and deterministic output
- status: local-pass

## Outcomes

- Validated contract precondition usage (`@require`) in CLI-style logic.
- Verified list-processing + summary paths compile and run in release-gate flow.
- Confirmed deterministic output suitable for automated smoke checks.

## Key Risks Observed

- Contract and effect declarations require up-front discipline in sample apps.
- Teams migrating from dynamic defaults may need stronger onboarding guidance.

## Follow-Up Backlog

- Add migration cookbook snippets focused on contract-driven CLI patterns.
- Expand lint diagnostics for common pilot-app anti-patterns.
