# VibeLang Module and Visibility Spec (v1.0 Target)

Status: normative target.

## Module Units

- A source file may declare module identity via:
  - `module qualified.name`
- If omitted, module identity is resolved by project path policy.

## Imports

- `import qualified.name` imports module namespace.
- Import resolution is deterministic and environment-independent under locked
  project mode.
- Ambiguous imports must produce deterministic diagnostics.

## Visibility Rules

- Declarations are module-private by default.
- `pub` marks declaration as exported from module boundary.
- Re-export syntax (if introduced) must be explicit and deterministic.

## Name Resolution

- Unqualified names resolve in this order:
  1. local scope
  2. module scope
  3. imported symbols according to explicit rules
- Shadowing is allowed with diagnostics policy where configured.

## Package Boundaries

- Modules belong to packages/workspaces defined by tooling manifests.
- Cross-package import requires manifest-level dependency declaration in locked
  mode.

## Cycles

- Import cycles are rejected unless explicit mutually-recursive module feature is
  introduced.
- Cycle diagnostics must include cycle path.

## Extension and Entry Rules

- `.yb` is canonical source extension.
- Entry-point resolution policy should be explicit (`main` module/function
  convention by tooling).

## Determinism Requirements

- Same source tree and lock state must yield identical import graph.
- Module resolution diagnostics must be stable.

## Deferred Notes

- Wildcard imports are discouraged and deferred unless explicit rule set is
  adopted.
