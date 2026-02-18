# VibeLang Charter

## Mission

VibeLang is a native programming language designed to combine:

- The simplicity of writing normally associated with scripting languages
- The runtime performance expected from systems languages
- Built-in concurrency ergonomics for modern multicore hardware
- Automatic memory management with predictable behavior
- Fast compile times that support a tight edit/build/test loop
- Intent-first development where correctness expectations live next to code

## Product Principles

1. **Simple to use, hard to misuse**
   - Common code should read naturally with low syntax noise.
   - Dangerous behavior must be explicit.

2. **Native performance by default**
   - AOT native code generation is the default path.
   - No mandatory VM in production deployment.

3. **Automatic memory management**
   - Memory is managed by a concurrent, generational GC.
   - Developers should not need manual free in application code.

4. **Concurrency as a core language capability**
   - First-class primitives for task spawning, channels, and select-like coordination.
   - Safe defaults over foot-guns.

5. **Fast development cycle**
   - Incremental compilation and parallel frontend/backend stages.
   - Predictable compile-time performance targets.

6. **Intent-first correctness**
   - Contracts such as `@intent`, `@examples`, `@require`, `@ensure`, and `@effect` are language features.
   - Contracts compile into deterministic checks and generated tests.

7. **AI-optional, deterministic core**
   - The compiler and runtime do not depend on AI services.
   - AI acts as an optional sidecar for suggestions and diagnostics.
   - AI linting is on-demand and must never be required for successful compilation.

8. **Low-cost operation**
   - Local-first tooling and indexer architecture.
   - Cloud AI usage is opt-in and budget constrained.

## Non-Negotiable Technical Requirements

These requirements are mandatory for v0.1 planning and v1.0 architecture:

- Native AOT executable output for Linux and macOS first
- Static type checking with strong local inference
- Automatic concurrent generational GC
- Structured concurrency primitives in core language and runtime
- Deterministic builds from pinned toolchain versions
- Contract annotations lowered to deterministic checks/tests
- AI features fully disable-able with no correctness impact
- AI intent linting must be non-blocking and local-first by default
- Strict AI latency and cost budgets with graceful fallback to non-AI tooling
- Incremental compilation design from day one

## Success Criteria

VibeLang is considered on track when it demonstrates:

- Beginner readability in core examples and standard library usage
- Predictable latency behavior under concurrent workloads
- Competitive performance on representative service and compute benchmarks
- Fast incremental compile latency in everyday development
- Strong signal from contract checks in CI and local development

## Intended User Segments

- Product engineers who need high performance without low-level ceremony
- Backend and infrastructure teams building concurrent services
- Data and AI platform teams that need safe, fast native components
- Learners who want systems-level output with approachable syntax

## Governance and Versioning

- Language changes must include:
  - A rationale document
  - Backward compatibility impact notes
  - Tooling and migration strategy
- Syntax-breaking changes are allowed only before v1.0 and require RFC approval.

## Decision Rule

When trade-offs conflict, choose in this order:

1. Correctness and determinism
2. Simplicity and readability
3. Performance
4. Compile-time speed
5. Expressiveness breadth

## Risk Mitigation Guardrails

- **Risk: AI feature creep bloats language/tooling**
  - Mitigation: keep AI in sidecar only, enforce strict cost/latency budgets, and require graceful degradation.
- **Risk: contract syntax becomes verbose or hard to trust**
  - Mitigation: keep v0.1 contract grammar intentionally small and prioritize deterministic, auditable checks first.
