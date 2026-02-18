# VibeLang AI Sidecar Cost Model (v0.1)

## Cost Goals

- Keep default usage near zero monetary cost
- Minimize CPU/RAM overhead on developer machines
- Make paid/cloud usage explicit, bounded, and transparent

## Default Mode

Default is local-only:

- Uses local semantic index and deterministic analyzers
- Optional lightweight local model for text suggestions
- No network calls unless user opts in

## Budget Controls

Configurable limits:

- Max cloud requests per day
- Max tokens per request/response
- Max monthly budget cap
- Per-project override and org policy

When limits are reached:

- Sidecar switches to local-only mode automatically.

## Latency Budgets

Soft target per suggestion request:

- Under 250 ms local
- Under 1.5 s cloud fallback

If budget exceeded, suggestion is skipped rather than blocking user workflow.

Intent lint budget policy:

- `vibe lint --intent --changed` is preferred default for local loops.
- Full-workspace intent lint is best for CI/nightly runs.
- If lint request exceeds latency budget, return partial results with explicit "incomplete" marker.

## Resource Budgets

Local runtime budgets:

- CPU utilization cap (background mode)
- RAM cap for sidecar process
- Optional "battery saver" mode on laptops

## Request Prioritization

Priority order:

1. Compiler diagnostics augmentations
2. On-demand intent lint for currently edited/changed functions
3. Intent/contract suggestion for active function
4. Refactor/perf suggestions for background analysis

Lower-priority work is dropped under load.

## Caching Strategy

- Cache by semantic hash + prompt template + model version
- Reuse previous suggestions when code context unchanged
- TTL-based invalidation

This reduces repeated inference cost in edit loops.

## Telemetry (Opt-In)

Collected only with explicit consent:

- Suggestion acceptance rate
- Latency metrics
- Error rates

No source upload telemetry by default.

## Team Policy Controls

Org-level policy can enforce:

- local-only mode
- approved model list
- network endpoint allowlist
- redaction requirements

## Cost Reporting

Provide visible usage dashboard:

- per-day requests
- estimated spend
- local compute time
- accepted vs ignored suggestions
