# VibeLang Mutability Model (v1.0 Target)

Status: partially implemented. Every rule below is marked IMPLEMENTED (enforced
by the compiler today, with a test), TARGET (not enforced), or DECIDED (a
design question that has been answered, with the answer recorded).

Implemented today: immutability by default for bindings and parameters, the
`mut` binding/parameter form, reassignment rejection (`E2110`), field mutation
through an immutable receiver (`E2111`), and in-place container mutation
through an immutable receiver (`E2112`).

Not implemented today, and therefore *not* guaranteed: `const` in any form,
per-field mutability in type declarations, typed local bindings
(`mut x: T := expr`), and any tracking of aliases — see "Known Escape Hatches"
below, which lists in one place every way a program can still mutate data
reached through an immutable binding. All of these are in `todo.md`.

## Principles

- Immutability by default.
- Mutation is explicit and auditable.
- Concurrency boundaries require stricter mutation rules.

## Binding Forms

- IMPLEMENTED — Immutable inferred binding:
  - `x := expr`
- IMPLEMENTED — Mutable inferred binding:
  - `mut x := expr`
- TARGET — Immutable constant binding:
  - `const x: T = expr`
  - `const` is not a keyword today; it lexes as an identifier and the form does
    not parse.
- TARGET — Type-annotated local binding (`x: T := expr`, `mut x: T := expr`).
  The grammar accepts only inferred locals; annotate the value instead. The
  `mut` form says so directly: `E1213` names the missing feature rather than
  reporting a generic syntax error.

## Reassignment Rules

- IMPLEMENTED — Reassignment (`=`) is allowed only for mutable bindings.
  Reassigning an immutable binding is `E2110`; the message names the binding
  and its declaration line and column.
- IMPLEMENTED — Rebinding with `:=` is legal and distinct from reassignment.
  A second `x := ...` introduces a fresh binding that shadows the previous one
  and carries its own mutability, so `mut x := 1` followed by `x := 2` leaves
  `x` immutable again. Scopes are flat in the checker: a binding introduced
  inside an `if`/`for` body stays visible after that body, which matches how
  the type environment already behaves.
- TARGET — `const` reassignment rejection, which needs `const` to exist first.

## Parameter Mutability

- IMPLEMENTED — Function and function-literal parameters are immutable by
  default; reassigning one is `E2110`.
- IMPLEMENTED — Mutable parameters are explicit: `mut arg: T`.
- IMPLEMENTED — `mut` is a declaration-site marker only. It grants the function
  body permission to reassign the parameter binding, to mutate fields through
  it, and to call an in-place container method on it; it does not change the
  calling convention, and there is no call-site form (`f(mut x)` is `E1213`).
- IMPLEMENTED — Parameter mutability does not bypass ownership/sendability
  constraints: the `go`/`thread` checks in `ownership.rs` are unchanged and run
  independently.

## Field, Index, and Container Mutation

- IMPLEMENTED — `obj.field = expr` requires a mutable receiver binding.
  Mutating through an immutable binding is `E2111`, whether the receiver is a
  local or a parameter. The root binding of the lvalue decides; chains
  (`a.b.c = expr`) are checked against `a`.
- IMPLEMENTED — In-place container mutation requires a mutable receiver
  binding. `xs.append(v)`, `xs.set(i, v)` / `m.set(k, v)`, and `m.remove(k)`
  mutate the receiver, so calling one through an immutable binding is `E2112`.
  These three are exactly the container operations the checker already tags
  with the `mut_state` effect on their receiver. The rest of the container
  surface was probed and does not mutate: `sort_desc` and `take` return fresh
  containers and leave the receiver untouched, and `get`/`contains`/`len` only
  read, so all of them stay ungated.
- IMPLEMENTED — `xs.set(i, v)` *is* VibeLang's index assignment, and it is
  gated. Brackets exist for *reads* (`xs[0]` parses and type-checks), but
  `xs[i] = expr` does not parse at all: an assignment target is an identifier
  or a `.field` chain, so `bs[0].v = 9` is a syntax error too. The method form
  is therefore the only index-assignment form there is, and the same holds for
  `m.set(k, v)` in place of `map[key] = value`. DECIDED: if a bracket
  assignment form is ever added, it must route through this same check rather
  than become a second, ungated path.
- IMPLEMENTED — Reaching a nested container still requires permission at the
  root, through either spelling: `xs.get(i).append(v)` and `xs[i].append(v)`
  are both checked against `xs`, because indexing and `.get(...)` hand back a
  view rather than a copy (measured: mutating the result is observable through
  `xs`).
- DECIDED, NOT DEFERRED — type declarations do not differentiate per-field
  mutability. There is no `mut field: T` form in a `type` declaration, and
  adding one is not planned: the receiver binding is the single place
  permission is recorded. The spec's earlier "field declared mutable (if type
  declaration differentiates mutability)" clause resolves to "it does not".
- DECIDED — channel operations (`ch.send(v)`, `ch.close()`, `ch.recv()`) do
  *not* require a `mut` binding. A channel handle is a communication endpoint
  shared across tasks by design, not a value whose contents the binding owns;
  requiring `mut ch := chan(1)` would make every `go`/`select` program declare
  a mutability it does not mean. Concurrent misuse stays the job of
  `docs/spec/ownership_sendability.md` (`E3201`-`E3203`).

## Known Escape Hatches

These are measured, not theoretical. Each one lets a program mutate data
reached through an immutable binding without any diagnostic, so the
immutability guarantee is real but **not** airtight:

- **Aliasing is not tracked.** `a := Box { v: 1 }` followed by `mut b := a` and
  `b.v = 99` compiles clean and changes `a.v`. The same holds for containers:
  `xs := [1, 2]`, `mut ys := xs`, `ys.append(3)` grows `xs`. `:=` copies the
  reference, not the value, and nothing propagates immutability across that
  copy. Closing this needs real reference/ownership tracking, which is a
  separate design (see `docs/spec/ownership_sendability.md` and `todo.md`).
- **Receivers with no binding root are unchecked.** `f().append(v)` is not
  reported: there is no binding at the root of the receiver to attribute the
  write to. (Reads through `.get(...)` and `[i]` *are* followed back to their
  root; free-function results are not.)
- **Passing to a `mut` parameter launders permission.** `f(xs)` where `f` takes
  `mut xs: List<Int>` lets the callee mutate a container the caller holds
  immutably. There is no call-site `mut`, so nothing at the call site records
  that.

## Const Semantics

TARGET in full. `const` does not exist in the lexer, parser, or checker.

- `const` values are immutable for program lifetime in their scope.
- `const` initializers MUST be compile-time evaluable.
- Taking mutable references to `const` values is illegal.

## Borrow/Reference Baseline

V1 target keeps reference model simple:

- No user-facing lifetime annotations required.
- Mutation through aliases in concurrent contexts is constrained by
  sendability/ownership checks.
- Mutation through aliases in *single-threaded* code is not constrained at all.
  `:=` copies the reference, so `mut b := a` yields a mutable view of an
  immutable `a`. This is the first entry under "Known Escape Hatches" and it is
  the main reason immutability here is a declaration-site discipline rather
  than a whole-program guarantee.
- Runtime synchronization primitives are required where shared mutation is
  allowed.

## Concurrency Interaction

- Mutable shared writes in concurrent contexts require explicit synchronization.
- Unsynchronized shared mutable writes are diagnostics errors in safe mode.
- Mutation across `go`/`thread`/async boundaries must satisfy
  `docs/spec/ownership_sendability.md`.

## Contracts and Mutability

- `@require` and `@ensure` are pure-expression contexts by default.
- Contract expressions cannot perform mutation.
- `old(expr)` snapshots read-only entry-time values.

## Diagnostics Requirements

Mutability diagnostics SHOULD include:

- IMPLEMENTED — which operation it was: reassignment (`E2110`), a field write
  through a receiver (`E2111`), or an in-place container call through a
  receiver (`E2112`, which also names the method)
- IMPLEMENTED — immutable binding target details: the binding name, its
  declaration line and column, and the exact form that would make it mutable
  (`mut x := ...` for a local, `mut x` for a parameter; loop variables and
  `select`/`match` pattern bindings have no `mut` form, so the message names
  the binder that bound them and says to bind a mutable copy instead)
- IMPLEMENTED — no contradictory pairs. A name bound by a `select` receive is
  reported only as immutable, never also as an unknown variable (`E2101`).
- TARGET — concurrency context in the message; `E3201`-`E3203` still report
  concurrency issues separately
- IMPLEMENTED — deterministic code and span, ordered by
  `Diagnostics::sorted()`

## Deferred Notes

- Fine-grained interior mutability primitives are deferred unless explicitly
  accepted in decision log.
