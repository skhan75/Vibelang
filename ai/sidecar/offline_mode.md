# VibeLang AI Sidecar Offline Mode (v0.1)

## Objective

Ensure VibeLang remains fully usable in disconnected or restricted environments.

Offline mode guarantees:

- Compiler and build tooling remain fully functional
- Semantic index and diagnostics continue to work
- Sidecar provides best-effort local suggestions only

## Activation

Offline mode can be enabled by:

- CLI flag: `--offline`
- Environment: `VIBE_OFFLINE=1`
- Auto-detection when network unavailable or policy disallows remote

## Behavior

When offline mode is active:

- Remote AI endpoints are never contacted.
- Cloud-only features are hidden/disabled.
- Suggestion flow uses local heuristic + local model if available.

## Feature Availability Matrix

| Feature | Offline |
| --- | --- |
| Parse/type diagnostics | yes |
| Contract checks | yes |
| Generated examples tests | yes |
| Symbol/index navigation | yes |
| Local AI suggestions | yes (if local model present) |
| Cloud inference | no |

## Local Suggestion Strategy

Without cloud:

- Use semantic index templates
- Rank suggestions via deterministic heuristics
- Optional local compact model for text polishing

## Security and Privacy

Offline mode is strongest privacy posture:

- No source egress
- No telemetry egress
- No remote model prompts

## UX Requirements

UI should clearly show:

- offline status active
- which suggestion features are reduced
- how to re-enable online mode

## Testing Offline Mode

Required tests:

- Build/test succeeds with sidecar disabled
- Sidecar startup in offline mode does not error
- No network calls occur while offline mode flag is set
- Suggestion API degrades gracefully
