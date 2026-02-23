# Chapter 9: Migration and Compatibility

Migration and compatibility are ongoing concerns in every language ecosystem.
This chapter explains how to migrate safely and how compatibility should be
managed in production teams.

## 9.1 Migration Philosophy

Migration in VibeLang aims to be:

- explicit,
- auditable,
- CI-verifiable.

The objective is not "silent auto-upgrade magic." The objective is controlled
adoption without hidden behavior change.

## 9.2 Extension Transition (`.vibe` -> `.yb`)

The canonical extension is `.yb`. During compatibility windows, older
extensions may still appear in repositories.

Guideline:

- standardize on `.yb` in new and modified files,
- avoid mixed-extension duplicates in the same logical module,
- run migration checks in CI.

## 9.3 Versioning and Compatibility Guarantees

Operational compatibility should be documented through:

- compatibility policy docs,
- release notes automation outputs,
- explicit breaking-change sections,
- deterministic upgrade/downgrade validation lanes.

Treat compatibility as a product surface, not only a compiler detail.

## 9.4 Practical Migration Steps

A typical migration flow:

1. inventory source files and extension usage,
2. convert extension and import/module references where needed,
3. run `vibe check` and `vibe test`,
4. run intent lint on changed paths,
5. run release-profile build and smoke tests,
6. attach migration evidence to PR/release notes.

## 9.5 Migration Safety Checklist

Before declaring migration complete, confirm:

- no extension conflicts remain,
- diagnostics are stable and expected,
- generated docs and indexes are clean,
- contract/example suites still pass,
- packaging/install paths remain valid.

## 9.6 Compatibility in API Evolution

When changing public APIs:

- preserve behavior where possible,
- when behavior changes, update intent/contracts/examples together,
- document breakages with clear migration guidance and timelines,
- include compatibility tests in the release gate.

This avoids "syntactically compatible but semantically broken" upgrades.

## 9.7 Compatibility Communication Discipline

Compatibility does not mean "everything is done." Mature releases publish known
limitations clearly.

In practice, this means documenting compatibility behavior, migration impact, and
breaking-change handling in user-facing release communication. This improves
trust because users can make informed decisions early.

## 9.8 Migration Example: Minimal Service

```txt
pub main() -> Int {
  @intent "boot service and report status"
  @effect io
  println("service ready")
  0
}
```

For migration, the function is simple. The process around it is not: extension,
module paths, tooling commands, and verification artifacts all matter.

## 9.9 Team Operating Model

A healthy migration operating model includes:

- one migration owner per subsystem,
- explicit rollback criteria,
- release notes sections pre-filled during migration PRs,
- evidence artifacts generated automatically where possible.

This model scales better than ad hoc conversion drives.

## 9.10 Clarification: Migration Notes Are About Stability, Not Fault

When this chapter discusses migration risk, it is not implying that VibeLang is
fundamentally unstable. Migration guidance exists because disciplined language
projects make compatibility boundaries explicit. That explicitness helps teams
plan upgrades safely instead of discovering behavior deltas late in production.

In short: migration documentation is a sign of engineering maturity. It reduces
surprise and support cost, especially in organizations with strict release
governance.

## 9.11 Chapter Checklist

You should now be able to:

- execute a controlled `.yb` migration workflow,
- distinguish compatibility promise from implementation detail,
- define migration evidence requirements for CI/release,
- communicate breakages with concrete mitigation paths.

---

Next: Chapter 10 examines internals from parser to native runtime pipeline.
