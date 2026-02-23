# Chapter 4: Contracts and Executable Examples

Contracts are one of VibeLang’s defining capabilities. They allow your code to
carry both implementation and verification metadata in the same place.

This chapter explains each annotation deeply and shows how to write contracts
that stay high-signal over time.

## 4.1 Why Contracts Matter in VibeLang

Traditional codebases often split behavior description across:

- comments,
- test files,
- review checklists,
- wiki docs.

That separation tends to drift. VibeLang contracts reduce drift by colocating
intent, examples, and invariants with executable logic.

## 4.2 Contract Annotation Set

VibeLang supports:

- `@intent "..."` - function purpose statement,
- `@examples { ... }` - executable input/output narratives,
- `@require predicate` - preconditions,
- `@ensure predicate` - postconditions,
- `@effect effect_name` - side-effect declarations.

Placement rule:

> Contract annotations appear at the top of function bodies before executable
> statements.

## 4.3 `@intent`: State the Outcome

`@intent` should be short and concrete.

Good:

- "k largest numbers sorted descending"
- "charge payment and return approved receipt"

Weak:

- "does processing"
- "sort then map then filter"

You want "what outcome should hold," not implementation trivia.

## 4.4 `@examples`: Turn Docs Into Tests

Examples are executable, not decorative.

```txt
@examples {
  clamp_percent(0, 10) => 0
  clamp_percent(5, 10) => 50
  clamp_percent(10, 10) => 100
}
```

The compiler/tooling path lowers these into generated tests and includes them in
`vibe test` flows.

Coverage guidance for public functions:

- one happy path,
- one boundary case,
- one zero/empty case (where meaningful).

## 4.5 `@require`: Guard Entry Conditions

Preconditions validate assumptions at function entry.

```txt
@require total > 0
@require payment.currency == "USD"
```

If preconditions fail:

- dev/test profiles treat it as hard failure with diagnostics,
- release behavior is profile policy driven but must remain deterministic.

## 4.6 `@ensure`: Validate Exit Guarantees

Postconditions run before return:

```txt
@ensure . >= 0
@ensure len(.) <= len(xs)
```

Special postcondition forms:

- `.` means the return value,
- `old(expr)` captures entry-time value for comparison.

Example:

```txt
@ensure from.balance == old(from.balance) - amount
@ensure to.balance == old(to.balance) + amount
```

## 4.7 `@effect`: Declare Side-Effect Budget

Base effect vocabulary:

- `alloc`
- `mut_state`
- `io`
- `net`
- `concurrency`
- `nondet`

Use effects to expose real behavior boundaries. Over-declaring everything
removes signal; under-declaring hides risk.

## 4.8 Full Contracted Function Example

```txt
pub settle(payment: Payment) -> Receipt {
  @intent "charge payment and return approved receipt"
  @examples {
    settle(valid_small_usd_payment()) => approved_small_receipt()
  }
  @require payment.amount > 0
  @require payment.currency == "USD"
  @ensure .status == "approved"
  @effect io

  result := gateway.charge(payment)
  result
}
```

This function communicates:

- business intent,
- input assumptions,
- expected output property,
- operational side effects.

## 4.9 Contract Determinism Rules

To keep contracts reproducible:

- contract expressions should avoid nondeterministic calls,
- contract contexts cannot perform I/O,
- random/time dependencies should be excluded unless explicitly modeled.

The objective is stable verification under stable inputs.

## 4.10 Style Guardrails for Large Codebases

Over-contracting can make code noisy. Use these practical guardrails:

1. One high-quality `@intent` line.
2. A few high-signal `@examples`, not dozens of near-duplicates.
3. Minimal `@ensure` lines that capture true invariants.
4. `@effect` entries that materially help review and analysis.

## 4.11 Anti-Patterns

Avoid:

- giant integration scenarios inside `@examples`,
- vague intents that cannot be checked,
- contracts that restate trivial implementation detail,
- postconditions that are effectively always true and provide no safety value.

## 4.12 Failure Diagnostics and Debugging

A high-quality contract violation should tell you:

- which function failed,
- which contract line failed,
- the relevant values,
- exact source location.

That detail is crucial for fast incident triage and for AI-generated code review
workflows where intent mismatch is the main risk.

## 4.13 Contract-First Refactoring Workflow

For risky refactors:

1. tighten `@intent` text,
2. add/upgrade boundary examples,
3. keep invariants explicit in `@ensure`,
4. refactor implementation,
5. run `vibe test` and `vibe lint --intent --changed`.

This workflow catches semantic drift earlier than test-only approaches.

## 4.14 What Contracts Are Not

Contracts are sometimes misunderstood as "extra comments with syntax." In
VibeLang, they are stronger than comments but still not a full formal-proof
system. They do not magically prove every property of your program. Instead,
they create executable, high-signal checkpoints around the behavior you care
about most.

A useful mental model is:

- intent states purpose,
- examples show representative behavior,
- preconditions constrain valid inputs,
- postconditions constrain valid outputs,
- effects expose operational boundaries.

When teams apply this model consistently, they get most of the practical value
of specification-oriented development without the overhead of heavyweight formal
methods on every function.

---

Next: Chapter 5 explains effects, cost reasoning, and performance visibility.
