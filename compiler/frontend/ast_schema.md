# AST Schema Notes (Phase 1)

This document captures the Phase 1 AST contract implemented in `vibe_ast`.

## Top-Level

- `FileAst`
  - `module: Option<String>`
  - `imports: Vec<String>`
  - `declarations: Vec<Declaration>`

## Declarations

- `Declaration::Function(FunctionDecl)`

## FunctionDecl

- `is_public: bool`
- `name: String`
- `params: Vec<Param>` — each `Param` carries `name`, `ty`, `is_mut` (set by the
  `mut a: T` form), and `span` (the parameter name, used by mutability
  diagnostics to point at the declaration)
- `return_type: Option<TypeRef>`
- `contracts: Vec<Contract>`
- `body: Vec<Stmt>`
- `tail_expr: Option<Expr>`
- `span: Span`

## Contracts

- `Intent`
- `Examples`
- `Require`
- `Ensure`
- `Effect`

## Statements

- `Binding`, `Assignment`, `Return`, `ExprStmt`
- `For`, `If`, `While`, `Repeat`, `Select`, `Go`

`Binding` carries `is_mut: bool`, set by the `mut name := expr` form. It is the
only place binding mutability is recorded; see `docs/spec/mutability_model.md`.

## Expressions

- literals (`Int`, `Float`, `Bool`, `String`, `List`, `Map`)
- `Ident`, `Member`, `Call`, `Binary`, `Unary`, `Question`
- contract helpers: `DotResult`, `Old`

## AST Stability Rule

Any schema-breaking changes in Phase 1 must update:

- this schema document
- parser tests/snapshots
- diagnostics expectations where affected
