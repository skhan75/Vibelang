# VibeLang Intent Spec (v0.1 Draft)

## Why Intents Exist

`@intent` captures purpose in a form that:

- Humans can quickly scan during review
- Tools can index for search, drift detection, and diagnostics
- Generated docs can surface without duplicating comments

## Intent Shape

Syntax:

```txt
@intent "short statement of expected behavior"
```

Recommended style:

- One sentence
- Concrete outcome
- Avoid implementation details

Good:

- `"k largest numbers, sorted desc"`
- `"move funds safely between accounts"`

Bad:

- `"sort then slice and return"`
- `"does stuff"`

## Intent Taxonomy (v0.1 Metadata)

While only free-text is required, tooling can classify text into tags:

- Correctness (`ordering`, `bounds`, `invariant`)
- Performance (`complexity`, `allocation`)
- Safety (`no_data_race`, `input_validation`)
- Domain (`payment`, `auth`, `query`)

These tags are derived by indexer/sidecar and do not alter core semantics.

## Intent and Contract Relationship

- `@intent` states **what** should happen.
- `@require/@ensure/@examples` state **how to verify** it.
- `@effect` states **what side effects are expected**.

Minimal complete pattern:

```txt
@intent "k largest numbers, sorted desc"
@examples {
  topK([3,1,2], 2) => [3,2]
}
@ensure sorted_desc(.)
```

## Intent Drift

Intent drift means implementation behavior no longer matches stated purpose.

Detection strategy:

1. Contract failures in generated checks
2. Semantic index change analysis (call graph and data-flow deltas)
3. Optional sidecar warnings when intent text and behavior diverge

## AI Intent Lint (On-Demand, Optional)

AI-based intent linting is recommended as an optional, on-demand workflow.

Proposed command surface:

- `vibe lint --intent`
- `vibe lint --intent --changed`

Design rules:

- Never required for compilation or correctness.
- Runs on top of semantic index and contract/effect metadata.
- Returns advisory diagnostics with confidence scores and rationale.
- Defaults to local-first execution; cloud inference is explicit opt-in.

Typical lint findings:

- Intent text likely too vague for verification
- Intent appears inconsistent with observed call/data-flow
- Intent claims missing matching checks/examples/effects
- Intent contradicts detected side effects (for example, claims pure but mutates state)

## Linting Guidelines

V0.1 lints (non-blocking by default):

- Missing `@intent` on exported function
- Overly vague intent text
- Intent present but no executable examples on public APIs
- Intent likely drifted from implementation (AI-assisted, on-demand)

## Trust and Gating Policy

- AI intent lint diagnostics are warnings by default.
- Teams may gate CI only on high-confidence findings and only in dedicated lint jobs.
- AI lint output must include evidence references (symbols, calls, effects, or contracts) for auditability.

## Cost and Performance

`@intent` metadata itself is low-cost:

- Stored in index and optional debug symbols
- No runtime overhead unless linked to checks in debug profile

For AI intent lint:

- Use semantic hash caching to avoid repeat analysis in edit loops.
- Drop low-priority or slow lint requests instead of blocking dev workflow.
