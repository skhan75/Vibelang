# VibeLang Ownership and Sendability (Phase 3 Hybrid Model)

## Goal

Phase 3 adopts a hybrid safety model:

- compile-time sendability and aliasing checks for common unsafe patterns
- explicit synchronization boundaries for shared mutable state
- no full borrow-checker/lifetime system in this phase

## Core Rules

## 1) Sendability for `go`

Values captured or passed into `go` task calls must be sendable:

- sendable: `Int`, `Float`, `Bool`, `Str`, `List<T>` where `T` is sendable
- not sendable in Phase 3 baseline: `Map<K,V>`, unknown dynamic values, and values containing non-sendable members

If a non-sendable value is passed to `go`, compilation fails with ownership diagnostics.

## 2) Shared Mutable State in Concurrent Context

If a function performs concurrency operations (`go`/`select`/channel ops), direct shared mutable writes should be explicit and auditable.

Phase 3 baseline check:

- member/field assignment in concurrent contexts is flagged unless synchronized through explicit runtime primitives

This catches high-risk race patterns early, even before a full ownership engine exists.

## 3) Channel Transfer Boundary

Channel send/recv are synchronization boundaries:

- sending transfers value visibility to receiver
- receiving observes the send-side write order for transferred values

## 4) Effect Coupling

Ownership/sendability and effects are coupled:

- concurrency behavior must be represented by `@effect concurrency`
- mutable shared writes in concurrent contexts must declare `@effect mut_state`
- transitive calls propagate effect requirements through call graph

## Non-Goals in Phase 3

- full Rust-style borrow checker
- user-visible lifetime annotations
- complete static race elimination for all dynamic patterns

These are deferred to later phases.
