# Chapter 14: Modules, Imports, and Visibility

Modular structure is where language design meets team architecture. This chapter
covers VibeLang’s module model and practical package composition guidance.

## 14.1 Module Units

A file may declare module identity:

```txt
module app.payments.settlement
```

If omitted, module identity follows project path policy as defined by tooling.

Be consistent. Mixed implicit/explicit style in one codebase can reduce
navigability and increase accidental import drift.

## 14.2 Imports

Import syntax:

```txt
import app.core.money
import app.core.errors
```

Import resolution should be deterministic under locked project mode.

## 14.3 Visibility Rules

By default, declarations are module-private. `pub` exports symbols across module
boundaries.

```txt
pub type Receipt { ... }
pub settle(payment: Payment) -> Result<Receipt, Error> { ... }
```

A useful discipline:

- keep internals private by default,
- expose small, stable API surfaces,
- document exported behavior with intent/contracts/examples.

## 14.4 Name Resolution Order

Unqualified name resolution follows deterministic precedence:

1. local scope,
2. module scope,
3. imported symbols.

Shadowing may be legal, but use it sparingly in public-facing code.

## 14.5 Package Boundaries

Modules live inside package/workspace boundaries defined by manifests and
tooling. Cross-package imports should align with explicit dependency
declarations, especially in locked mode.

This matters for reproducibility and supply-chain governance.

## 14.6 Cycle Handling

Import cycles are rejected unless an explicit feature introduces a
mutually-recursive module model.

Cycle diagnostics should include the cycle path, making refactors easier.

## 14.7 Designing Stable Module APIs

For stable module APIs:

- keep exported types small and explicit,
- avoid leaking internal helper types,
- expose `Result`-typed boundaries for recoverable failures,
- preserve behavior contracts across upgrades.

## 14.8 Example Module Layout (Service)

```txt
module app.invoice

import app.invoice.model
import app.invoice.policy
import app.core.errors

pub create_invoice(input: model.InvoiceInput) -> Result<model.Invoice, errors.DomainError> {
  @intent "build invoice from validated input"
  @require policy.valid_input(input)
  @ensure .is_ok()
  @effect alloc

  ...
}
```

This gives clear ownership boundaries and API intent.

## 14.9 Migration and Modules

During extension or package migrations:

- verify module declarations and import paths together,
- run full `check/test/lint` cycles after path changes,
- include compatibility notes for moved/renamed modules.

Module-path drift is a common migration failure source; automate checks where
possible.

## 14.10 Visibility and Concurrency

Concurrency-safe design often improves when mutable internals stay private and
only controlled APIs are public. Public exports should avoid exposing raw shared
mutable internals that make sendability/race guarantees harder to maintain.

## 14.11 Documentation and Discoverability

Module quality is not just compile success. High-quality module boundaries:

- have clear exported function intent,
- provide example usage,
- produce stable generated docs,
- keep naming coherent with domain terms.

This increases onboarding speed and reduces misuse.

## 14.12 Common Mistakes

1. exporting too much by default,
2. importing broad internal modules instead of narrow APIs,
3. allowing naming collisions that hide symbols unexpectedly,
4. mixing migration concerns with unrelated module refactors in one step.

## 14.13 Clarification: Module Design Is an Operational Concern

Module and visibility decisions are often treated as "just code organization."
In practice they directly affect release risk, refactor cost, and incident
triage speed. Clear boundaries reduce blast radius, improve API trust, and make
compatibility promises easier to uphold.

When teams invest in coherent module architecture early, they gain compound
benefits in documentation quality, test targeting, and long-term migration
safety.

## 14.14 Chapter Checklist

You should now be able to:

- structure modules with clear visibility boundaries,
- manage imports deterministically,
- design stable public APIs,
- avoid cycle and naming-resolution pitfalls.

---

Next: Chapter 15 covers ownership, sendability, and memory semantics.
