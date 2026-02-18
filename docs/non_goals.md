# VibeLang Non-Goals (v0.1 to v1.0)

This document lists capabilities that VibeLang intentionally does **not** target in early phases.

## Language Non-Goals

- **No dynamic runtime type system as primary model**
  - VibeLang is statically typed with inference, not dynamically typed by default.

- **No hidden exceptions model for core control flow**
  - Error behavior should remain explicit and analyzable.

- **No manual memory management in normal application code**
  - Manual free-like APIs are not part of everyday workflow.

- **No syntax mimicry as a goal**
  - VibeLang borrows good ideas from modern languages but does not try to copy one language's syntax.

- **No "everything is magic" metaprogramming in v0.1**
  - Keep compiler behavior transparent and debuggable.

## Runtime and Performance Non-Goals

- **No hard real-time guarantees in v0.1**
  - The GC is optimized for low pauses, not strict hard real-time systems.

- **No "single fastest benchmark language" claim**
  - Goal is practical high performance with safer defaults, not benchmark gaming.

- **No universal zero-allocation guarantee**
  - Allocation is allowed; contracts and profiling expose it where relevant.

## Concurrency Non-Goals

- **No unrestricted shared mutable memory by default**
  - Coordination should favor channels/message passing and structured patterns.

- **No user-managed scheduler internals in v0.1**
  - The runtime scheduler remains implementation-defined with stable user-facing primitives.

## AI Non-Goals

- **No AI-dependent compilation**
  - Build correctness cannot depend on cloud inference or model availability.

- **No AI lint gate in compile path**
  - AI intent lint warnings must not block parsing, type checking, codegen, or linking.

- **No mandatory paid AI features**
  - Core compiler and language tooling stay usable offline and free.

- **No automatic code mutation without explicit user action**
  - AI suggestions are advisory; users approve any changes.

## Tooling Non-Goals

- **No monolithic IDE lock-in**
  - Language tooling should expose open CLI/LSP interfaces.

- **No unstable plugin API commitment in v0.1**
  - Tooling internals may evolve rapidly before v1.0.

## Scope Non-Goals

- **No attempt to solve every domain in first release**
  - Prioritize backend services, concurrent systems, and developer productivity.

- **No full self-hosting requirement for initial milestones**
  - Self-hosting is a later objective after core semantics and runtime stability.
