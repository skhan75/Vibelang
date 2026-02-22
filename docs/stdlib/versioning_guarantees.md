# Stdlib Versioning Guarantees (v1.x)

## Compatibility contract

- **Stable APIs**: no breaking signature changes in `v1.x`.
- **Preview APIs**: may evolve in minor releases with release-note callouts.
- **Experimental APIs**: no compatibility guarantee.

Tier assignment source of truth: `stdlib/stability_policy.md`.

## Behavior guarantees

- Stable + preview APIs must provide deterministic behavior for equal inputs unless explicitly
  marked nondeterministic.
- Error behavior must be documented (boolean sentinel, fallback value, or panic).

## Deprecation policy

- Stable API removals require:
  - deprecation warning period in prior `v1.x` release,
  - migration guidance in docs,
  - release-note entry.
- Preview API removals require:
  - release-note callout,
  - compatibility note in module README.
