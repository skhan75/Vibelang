# Pilot Case Study: Service Reference

- pilot: `pilot-apps/service_reference/main.yb`
- class: concurrent service-style batch processing
- status: local-pass

## Outcomes

- Validated `chan` + `go` fan-out/fan-in flow on deterministic fixture input.
- Confirmed debug/profile/unsafe/allocation artifacts emit on build path.
- Captured operator-facing output for repeatable RC smoke validation.

## Key Risks Observed

- Missing effect declarations fail fast (good safety, but can slow iteration).
- Channel sendability diagnostics require explicit typing on ambiguous values.

## Follow-Up Backlog

- Improve quick-fix hints for missing `@effect` declarations.
- Add richer worker-pool examples to docs and starter templates.
