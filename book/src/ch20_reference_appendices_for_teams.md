# Chapter 20: Reference Appendices for Teams

This final chapter is a compact reference kit for teams who want to operationalize
VibeLang standards in real projects.

Unlike earlier chapters, this one is designed to be scanned repeatedly during
implementation, review, and release cycles.

## 20.1 Daily Developer Checklist

Use this at the end of each meaningful change:

1. Does function/module intent still match behavior?
2. Are contracts high-signal and not redundant?
3. Are effects accurate (`alloc`, `io`, `concurrency`, etc.)?
4. Are concurrency boundaries explicit and safe?
5. Do check/test/lint loops pass?
6. Is release-profile behavior understood?

If any answer is uncertain, pause and tighten the code before merge.

### How to Interpret Checklist Language

This chapter intentionally compresses guidance into checklist form because it is
meant for day-to-day operational use. The checklists are not implying that
VibeLang is currently failing these areas; they define the discipline teams
should apply to preserve correctness and determinism as systems evolve.

When using these appendices, treat each checklist item as a discussion starter.
If a team cannot explain an item in prose during review ("why this effect is
declared," "why this boundary is safe," "why this release lane is required"),
that is a signal to add documentation or tighten design clarity.

## 20.2 Public API Checklist

For each new or changed `pub` API:

- explicit signature present,
- clear `@intent`,
- minimal `@examples` covering happy/boundary/empty cases,
- `@require` and `@ensure` represent real invariants,
- effect declarations reflect actual behavior,
- error channels are stable and documented.

This checklist significantly reduces long-term maintenance cost.

## 20.3 Contracts Quality Rubric

Grade each function quickly:

### A (excellent)

- one clear intent line,
- precise pre/post conditions,
- examples encode behavior boundaries,
- no redundant checks.

### B (acceptable)

- intent mostly clear,
- contracts useful but could be sharper,
- examples cover common paths but miss edge cases.

### C (needs work)

- vague intent,
- trivial/duplicative contracts,
- no useful examples on important APIs.

Apply rubric in code review comments to keep quality standards consistent.

## 20.4 Effect Declaration Rubric

For every effect line ask:

- Is this effect truly present?
- Is any significant effect missing?
- Is this function now less/more effectful than before?

Desired state:

- no hidden behavior,
- no noisy over-declaration,
- clear transitive expectations in call graph.

## 20.5 Concurrency Safety Checklist

Before merging concurrent code:

1. what values cross `go`/`thread`/channel/async boundaries?
2. are those values sendable?
3. where is ownership transferred?
4. where is shared mutation synchronized?
5. where are timeout and cancellation semantics defined?
6. what is failure propagation policy?

If these are not explicit in code, they are likely implicit bugs.

## 20.6 Module Boundary Checklist

- module name and file layout coherent,
- imports explicit and deterministic,
- `pub` exports minimal and intentional,
- internal helpers private,
- no accidental cycle creation,
- migration impact assessed for renamed/moved symbols.

Large systems degrade quickly when module boundaries are sloppy.

## 20.7 Error-Model Checklist

For each fallible path:

- Is `Result<T,E>` used for recoverable failures?
- Is `?` propagation intentional?
- Are error categories stable and actionable?
- Are contract failures used for invariants (not domain alternatives)?
- Are panic/trap paths truly unrecoverable?

This prevents mixed error semantics that confuse callers.

## 20.8 Performance and Cost Checklist

Before release:

- allocation-heavy paths identified (`@effect alloc` review),
- repeated string/container expansion audited,
- unnecessary task/channel churn eliminated,
- profile behavior validated (dev/test vs release),
- benchmark lanes reviewed for regressions.

Optimize for predictable behavior, not microbenchmark vanity.

## 20.9 Deterministic CI Template

A practical CI skeleton:

```bash
vibe fmt . --check
vibe check src/
vibe test src/
vibe lint . --intent --changed
vibe build src/main.yb --profile release
```

Add project-specific checks for:

- compatibility,
- packaging integrity,
- install smoke,
- reproducibility evidence.

## 20.10 Release Readiness Template

Use this markdown template in release PRs:

```txt
## Release scope
- modules changed:
- behavior changes:
- compatibility impact:

## Language/tooling validation
- check:
- test:
- lint --intent:
- release build:

## Contracts/effects changes
- new or updated @intent:
- new or updated @require/@ensure:
- effect declarations changed:

## Concurrency/memory/ownership impact
- boundary changes:
- sendability implications:
- synchronization changes:

## Known limitations and follow-ups
- ...
```

Templates reduce forgotten risk communication.

## 20.11 Incident Triage Starter

When production behavior diverges:

1. reproduce with deterministic input where possible,
2. classify failure channel (result/contract/trap),
3. inspect recent intent/contracts/effects changes,
4. inspect concurrency boundary and timeout changes,
5. compare release artifact metadata and toolchain versions.

This ties incident response back to language/tooling design signals.

## 20.12 Migration Pull-Request Template

For extension/module compatibility migrations:

```txt
## Migration intent
- what moved/changed:
- why now:

## Mechanical changes
- extension/path/module edits:

## Semantic checks
- behavior unchanged?:
- new contracts/examples?:

## Toolchain validation
- check/test/lint/build:

## Compatibility notes
- caller impact:
- rollback plan:
```

Standard templates prevent accidental omission of critical migration context.

## 20.13 Team Onboarding Path (2 Weeks)

A practical onboarding schedule:

- **Days 1-2:** Chapters 1-4 + small exercises,
- **Days 3-5:** Chapters 5-8 + contracts/effects review practice,
- **Week 2 (first half):** Chapters 9-15 + boundary-heavy coding tasks,
- **Week 2 (second half):** Chapters 16-19 + release-slice simulation.

This produces engineers who can contribute safely, not just compile code.

## 20.14 Quality Gates by Maturity Level

### Level 1 (early project)

- check + test + fmt.

### Level 2 (growing team)

- add intent lint and public-API contract expectations.

### Level 3 (production-critical)

- add reproducibility/packaging/install evidence lanes,
- enforce known-limitations and release-note discipline.

Choose level intentionally; do not over-constrain too early.

## 20.15 Suggested Team Standards Document

Write a lightweight team standards file that includes:

- naming conventions,
- required contract coverage for public APIs,
- allowed effect declaration patterns,
- concurrency design requirements,
- release checklist references.

When standards are written down, review quality becomes less subjective.

## 20.16 Final Reference Checklist

Before declaring a VibeLang feature "done," confirm:

- syntax and semantics are correct,
- intent and contracts are aligned,
- effects and cost implications are explicit,
- boundary safety is verified,
- release evidence is complete.

If all five are true, you are operating in VibeLang’s intended engineering mode.

---

This chapter closes the expanded edition of The VibeLang Book. Keep this
appendix nearby during implementation and review; it is designed for repeated
operational use.
