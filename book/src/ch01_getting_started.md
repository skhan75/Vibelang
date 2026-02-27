# Chapter 1: Getting Started and Mental Model

This chapter gives you a practical entry into VibeLang and introduces the core
mental model that repeats throughout the book.

If you remember only one idea, remember this:

> VibeLang is designed so that **speed, clarity, and correctness remain aligned**
> from local iteration to production release.

## 1.1 What VibeLang Is Optimizing For

VibeLang is a native-first language and toolchain that aims to
balance:

- deterministic AOT compilation,
- low-noise syntax and strong readability,
- explicit intent/contract semantics for behavior,
- practical concurrency primitives (`go`, `chan`, `select`, cancellation),
- release-governed operational trust (reproducibility, auditability, evidence).

Unlike many ecosystems where these concerns are distributed across separate
frameworks and custom policy scripts, VibeLang treats them as part of one
coherent programming model.

## 1.2 Install and Verify

The repository documents two main paths:

- packaged install (recommended for most users),
- source build (recommended for contributors/toolchain development).

For source build:

```bash
git clone https://github.com/skhan75/VibeLang.git
cd VibeLang
cargo build --release -p vibe_cli
export PATH="$PWD/target/release:$PATH"
vibe --version
vibe --help
```

Packaged install documentation is platform-specific under:

- `docs/install/linux.md`
- `docs/install/macos.md`
- `docs/install/windows.md`

## 1.3 Your First Program

```vibe
pub main() -> Int {
  @effect io
  println("hello from vibelang")
  0
}
```

Run it:

```bash
vibe run main.yb
```

This tiny program already shows two important VibeLang traits:

1. return type is explicit;
2. side effects are declared (`@effect io`).

## 1.4 Your First Toolchain Loop

A healthy local loop in VibeLang is:

```bash
vibe check main.yb
vibe run main.yb
vibe test main.yb
vibe fmt . --check
vibe lint . --intent --changed
```

This loop matters because VibeLang is intentionally **workflow-aware**: code,
contracts, and diagnostics are expected to evolve together.

## 1.5 Mental Model: Build Software in Five Layers

When designing code in VibeLang, think in layers:

1. **Behavior layer**: what your function should do.
2. **Verification layer**: how you assert that behavior (`@examples`,
   `@require`, `@ensure`).
3. **Effects layer**: what external interactions are allowed (`@effect`).
4. **Concurrency layer**: where work can run in parallel and how data moves
   safely.
5. **Release layer**: whether your build/test/lint/reproducibility evidence says
   the change is ready.

This layering is why VibeLang can support AI-assisted coding without surrendering
control. Suggestions are allowed to move quickly, but they still have to pass a
deterministic language/toolchain path.

## 1.6 Determinism as a Daily Discipline

In many teams, determinism is treated as a release concern only. In VibeLang,
it is a daily engineering discipline:

- stable diagnostics,
- predictable evaluation rules,
- explicit effect declarations,
- reproducible build policy,
- release-quality evidence.

The result is fewer "it worked locally" surprises and faster root-cause analysis
when failures happen.

## 1.7 Intent-Driven Development in One Minute

A simple function can carry purpose and checks:

```txt
pub clamp_percent(done: Int, total: Int) -> Int {
  @intent "return completion percentage clamped to [0, 100]"
  @examples {
    clamp_percent(0, 10) => 0
    clamp_percent(5, 10) => 50
    clamp_percent(10, 10) => 100
  }
  @require total > 0
  @ensure . >= 0
  @ensure . <= 100
  @effect alloc

  raw := (done * 100) / total
  if raw < 0 {
    0
  } else if raw > 100 {
    100
  } else {
    raw
  }
}
```

This is a key difference from comment-driven codebases:

- purpose is explicit,
- examples are executable,
- pre/post conditions are checkable,
- effect expectations are machine-readable.

## 1.8 Common Beginner Mistakes

1. **Forgetting `@effect` declarations** for visible behavior (especially `io`,
   `alloc`, `concurrency`).
2. **Using vague intents** ("does stuff") that cannot guide verification.
3. **Writing too many redundant contract lines** instead of a few high-signal
   guarantees.
4. **Skipping `vibe lint --intent` in change-heavy branches**, losing early drift
   detection value.
5. **Treating concurrency as syntax only** instead of data-movement and boundary
   safety design.

## 1.9 Detailed Walkthrough of the First Program

The hello-world sample may look trivial, but each line teaches a core VibeLang
idea:

- `pub main() -> Int` makes entry-point shape and return contract explicit.
- `@effect io` tells reviewers and tooling this function performs visible
  side effects.
- `println(...)` is the effectful action.
- terminal `0` makes success value explicit and deterministic.

This explicitness is intentional. In many ecosystems, equivalent semantics are
implicit, scattered, or hidden in conventions. VibeLang chooses explicit
declaration because it scales better for audits, AI-assisted refactors, and
release pipelines where hidden behavior becomes expensive.

## 1.10 Chapter Checklist

Before moving to Chapter 2, verify you can:

- install and run `vibe`,
- compile and execute a hello-world program,
- explain why `@effect` is valuable,
- describe the difference between intent and implementation,
- run the baseline toolchain loop (`check`, `run`, `test`, `fmt`, `lint`).

---

Next: Chapter 2 formalizes syntax and semantic foundations.
