# Chapter 16: ABI, FFI, and Unsafe Boundaries

VibeLang’s safe surface is intentionally strict. But real systems sometimes need
foreign interfaces and low-level boundaries. This chapter explains how to do that
without destroying determinism and auditability.

## 16.0 Quick Definitions (Beginner-Friendly)

Before we go deeper, here are plain-language definitions:

**ABI (Application Binary Interface)**  
ABI is the machine-level agreement for how compiled code talks to other compiled
code. It defines things like:

- how function arguments are passed (registers vs stack),
- how return values are returned,
- how struct fields are laid out in memory,
- alignment and padding rules,
- symbol/calling-convention expectations.

You can think of ABI as "the wiring format" between binaries.

**FFI (Foreign Function Interface)**  
FFI is the feature that lets VibeLang call code written in other languages (for
example C libraries), and in some cases allows foreign code to call into
VibeLang.

**Unsafe boundary**  
An unsafe boundary is a point where the compiler/runtime cannot fully enforce
all safety guarantees automatically, so human-written rules/wrappers must carry
that burden. Typical unsafe boundary risks include:

- wrong pointer lifetime assumptions,
- incorrect ownership transfer,
- incompatible struct layout,
- thread-affinity violations.

This does **not** mean VibeLang safe-surface code is broken. It means foreign
interop always requires explicit discipline.

## 16.1 Why ABI/FFI Matters

Most non-trivial production systems eventually need to integrate with native
components:

- performance-critical C/C++ libraries (compression, crypto, codecs),
- OS/platform SDKs,
- vendor-provided binary libraries,
- existing service agents written in another language.

That is where ABI/FFI matters. If you only write pure VibeLang code, the
language/runtime can enforce many invariants for you. The moment you cross into
foreign code, you must also satisfy binary-level contracts.

### Concrete beginner example

Imagine you want to call a C function:

```c
int32_t add_i32(int32_t a, int32_t b);
```

From VibeLang, you might expose a wrapper. If your wrapper assumes a different
argument size/order than what the C side expects, the program can return wrong
values or crash, even though your VibeLang logic "looks fine." That mismatch is
an ABI contract failure, not a normal type-checking mistake inside one language.

### Why VibeLang calls this an explicit boundary

VibeLang documentation emphasizes explicitness at this boundary so teams do not
rely on accidental behavior. You should document:

- expected foreign function signatures,
- ownership/lifetime rules,
- thread and error propagation expectations,
- target/platform assumptions.

Treating FFI as "just another function call" is one of the fastest paths to
hard-to-debug production failures.

## 16.2 ABI Baseline (What VibeLang Assumes)

VibeLang uses a conservative ABI baseline:

1. By default, external boundaries follow platform C ABI unless explicitly
   configured otherwise.
2. Primitive type sizes and layout assumptions must align with VibeLang numeric
   and ABI specifications.
3. Struct/record field layout, alignment, and padding must be deterministic for
   the target platform.
4. Any binary compatibility claim must name the exact target triple(s), because
   ABI details can differ across targets.

### Why target triples matter

"Works on my machine" is not enough for ABI claims. A Linux x86_64 build and a
Windows x86_64 build may differ in calling convention or layout expectations for
specific boundary types. If you publish a native integration, you should state
the supported target triples explicitly and test those boundaries directly.

### Practical wrapper mindset

When writing ABI/FFI wrappers, prefer small, explicit, well-tested boundary
functions over broad direct interop across your codebase. This keeps unsafe
assumptions localized and auditable.

## 16.3 String and Buffer Interop

This section is **not** saying "VibeLang currently has a string-memory bug."
It is describing a general FFI risk class that every systems language must
handle explicitly.

Inside pure VibeLang code, `Str` safety and lifetime behavior are governed by
the language runtime model. The risk appears when crossing into foreign code
because the foreign side may not share VibeLang's safety assumptions.

When we say string representation must be explicit, we mean both sides of the
boundary must agree on concrete wire/layout semantics, such as:

- pointer + length pairs,
- encoding expectations (UTF-8, or explicitly converted form),
- mutability expectations (read-only vs writable),
- who owns allocation and who is allowed to free memory.

Ownership mode must also be explicit at every boundary call:

- **borrow**: foreign code can read temporary data but cannot keep or free it,
- **copy**: foreign code receives an independent copy and owns that copy's
  lifetime,
- **move/transfer**: ownership is transferred, and responsibility for release is
  clearly reassigned.

If these rules are implicit instead of explicit, teams can introduce boundary
bugs such as use-after-free, double-free, or stale-pointer reads in foreign
layers. So this guidance is preventive design discipline, not a statement that
VibeLang safe-surface string handling is broken.

### Concrete mental model

If you pass a VibeLang `Str` to a C API, do not rely on assumptions like "the
C side probably copies it." Either wrap with a function that guarantees copy
semantics, or document and enforce a borrow contract with strict lifetime
boundaries. In production code, ambiguity at this boundary should be treated as
a release blocker.

## 16.4 Error Interop

Foreign failures should map deterministically into VibeLang channels:

- `Result<T,E>` where practical,
- stable categorized errors where full fidelity is not possible.

Do not silently collapse all foreign failures into one opaque category unless you
have no better option.

## 16.5 Threading and Reentrancy Boundaries

FFI and threading require care:

- foreign calls from runtime threads must satisfy thread-safety contract,
- callbacks into VibeLang must respect scheduler/GC safety points,
- thread-affine APIs should be called through clearly documented wrappers.

## 16.6 Unsafe by Default at FFI Boundary

FFI is considered unsafe by default unless wrapped in validated safe
abstractions.

That means production code should expose small safe wrapper APIs and keep raw
boundary calls isolated and reviewed.

## 16.7 Unsafe Escape Hatch Policy

VibeLang defines strict unsafe markers:

```txt
// @unsafe begin: <reason>
// @unsafe review: <ticket-or-change-id>
// @unsafe end
```

Policy requirements include:

- non-empty reason,
- mandatory review reference,
- matching begin/end,
- no nested unsafe blocks.

## 16.8 Build-Time Unsafe Audit Artifact

Build flows emit unsafe audit artifacts. Violating marker policy should fail
builds. This is a major operational advantage: unsafe usage becomes searchable,
reviewable, and release-validated.

## 16.9 Designing Safe Wrappers

A good wrapper should:

- validate argument shape and ranges,
- enforce ownership/lifetime policy,
- convert foreign error surfaces into stable internal forms,
- minimize unsafe region size.

Keep the unsafe region tiny and heavily documented.

## 16.10 Determinism Rules Still Apply

Unsafe and FFI do not exempt code from determinism expectations:

- diagnostics remain stable,
- audit artifacts remain reproducible for identical inputs/toolchain,
- release evidence remains required.

This preserves trust even at low-level boundaries.

## 16.11 Practical Review Checklist

For any unsafe/FFI change:

1. Is boundary need justified?
2. Are markers complete and policy-compliant?
3. Is review evidence linked?
4. Is wrapper narrow and explicit?
5. Are failure and ownership semantics tested?
6. Are reproducibility/audit artifacts generated and checked?

## 16.12 Common Mistakes

1. broad unsafe regions around large logic blocks,
2. missing `@unsafe review` linkage,
3. exposing raw foreign handles broadly in application code,
4. unclear ownership transfer at boundary,
5. assuming "works in one build" implies ABI stability.

## 16.13 Clarification: Boundary Guidance Is Preventive, Not Defect Reporting

Throughout this chapter, risk language ("unsafe," "boundary hazard," "memory
risk") describes classes of integration mistakes teams should prevent. It is not
a claim that the VibeLang safe surface is currently violating those guarantees.

This distinction is important for documentation tone: we describe boundary
failure modes in detail so teams can avoid them before incidents happen.

## 16.14 Chapter Checklist

You should now be able to:

- reason about ABI and FFI boundary design in VibeLang,
- apply unsafe marker policy correctly,
- build safer wrapper patterns for foreign APIs,
- preserve determinism and auditability at low-level edges.

---

Next: Chapter 17 covers deterministic builds, profiles, and release engineering.
