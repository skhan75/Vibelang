# Preface: How to Read This Book

Welcome to **The VibeLang Book**.

This book is written as a full technical guide for people who want to use
VibeLang seriously: for prototypes, production services, agentic workflows, and
release-governed software delivery.

VibeLang sits at a specific intersection:

- native AOT performance,
- low-noise authoring ergonomics,
- explicit intent and contracts,
- deterministic build and diagnostics behavior,
- practical concurrency and async models.

This means the language is not only about "how to write syntax"; it is also
about **how to maintain correctness under change**, especially in teams using
AI-assisted development.

## What This Book Covers

The chapters are organized from practical to deep:

1. You begin with setup, first programs, and mental model.
2. You then learn syntax, types, literals, control flow, modules, and core data
   structures.
3. Next comes VibeLang’s signature layer: `@intent`, `@examples`, `@require`,
   `@ensure`, and `@effect`.
4. Finally, you move into ownership/sendability, memory model, FFI boundaries,
   deterministic releases, and long-term engineering practices.

The goal is not only to help you write code that compiles; it is to help you
write code that remains auditable, reproducible, and maintainable as systems
scale.

## Who This Book Is For

This book is designed for multiple audiences:

- **Application developers** moving from dynamic or general-purpose languages
  into native, deterministic systems.
- **Platform and infra teams** who need reproducibility, policy-aware builds,
  and explicit runtime guarantees.
- **AI-native engineering teams** that use copilots or autonomous codegen but
  still need high confidence in behavior.
- **Language/tooling contributors** who want to understand how the compiler,
  runtime, and governance constraints are connected.

You do not need deep compiler knowledge to start. We introduce concepts in
layers and call out where implementation details become relevant.

## Normative vs Explanatory Material

Like most serious language ecosystems, VibeLang has both:

- **Normative specification artifacts** (what the language and runtime
  guarantee), and
- **Explanatory guides** (how to think about and apply those guarantees).

This book is explanatory and practical, but it is written to stay aligned with
the normative spec suite.

When there is ambiguity, use this precedence order:

1. the current grammar file under `docs/spec/`
2. normative model docs (`type_system`, `numeric_model`, memory/concurrency/ABI)
3. `docs/spec/syntax.md` and `docs/spec/semantics.md`
4. examples and tutorials

That ordering ensures teams do not accidentally treat convenience examples as
language law.

## The VibeLang Mindset

VibeLang works best when you develop with these habits:

- State intent close to code.
- Keep contracts executable.
- Prefer deterministic, observable workflows over hidden magic.
- Treat concurrency as a first-class design decision, not an afterthought.
- Use tooling (`check`, `test`, `lint --intent`, release workflows) as part of
  design, not only as final QA.

In short: write code that is fast, readable, and trustworthy under continuous
change.

## About AI in VibeLang

VibeLang deliberately separates:

- **AI as assistance** (intent linting, guidance, suggestion), and
- **compiler/runtime correctness** (deterministic, local, reproducible).

This is a key architectural principle. AI can help you move faster, but your
core correctness path should not depend on AI availability or model behavior.

## How to Use This Book Effectively

Recommended reading modes:

- **First pass:** read Chapters 1 through 8 in order.
- **Production pass:** jump to Chapters 13 through 18 for production engineering,
  determinism, and advanced boundaries.
- **Reference mode:** use chapters independently while implementing features.

Practical tip: when a chapter introduces a concept, run the examples locally and
adapt them into a tiny real module in your own project. VibeLang concepts
settle quickly when you see the full toolchain loop (`check` -> `build` ->
`run` -> `test` -> `lint`).

## Example Conventions in This Book

Most examples are aligned with documented language forms. Some examples use
domain helper calls (for example `parse_i64`, `gateway.charge`, or
`approved_small_receipt`) to focus on language concepts. Treat those helper
symbols as application-level placeholders unless a chapter says the snippet is
complete and runnable as-is.

## On Brevity vs Depth

You will see both prose and compact reference lists in this book. The lists are
meant as scanning aids, not as replacements for explanation. When a section
looks concise, read it as an index into deeper semantics that are expanded in
later paragraphs or linked normative docs.

To avoid ambiguity, interpret chapter content with this rule: short list items
name a guarantee or concept, while surrounding text explains the operational
meaning, trade-offs, and failure boundaries. If a sentence sounds like a warning
("this can fail" or "this is risky"), that usually refers to boundary misuse in
real systems, not an implicit claim that VibeLang safe-surface behavior is
currently broken.

## Scope

This book is the public product guide for VibeLang. It focuses on what the
language provides and how to use it effectively in real projects.

Internal release tracking artifacts, milestone notes, and planning discussions
belong in engineering documentation and are intentionally out of scope here.

When behavior depends on environment, profile, or platform, the guide explains
that dependency directly in practical terms.

---

If you are ready, continue to Chapter 1.

