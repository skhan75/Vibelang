# VibeLang Syntax Spec (v0.1 Draft)

## Goals

Syntax in v0.1 is designed for:

- Low typing friction
- High readability
- Explicit correctness metadata next to implementation

## Phase 1 Grammar Freeze

- Parser source of truth: `docs/spec/grammar_v0_1.ebnf`
- Resolved ambiguities appendix: `docs/spec/phase1_resolved_decisions.md`
- The EBNF in this document is explanatory; implementation should follow the freeze artifact.

## Source File Layout

A file may contain:

1. Optional module declaration
2. Zero or more imports
3. Type/function declarations

Example:

```txt
module app.math

import std.math

topK(xs, k) {
  @intent "k largest numbers, sorted desc"
  @examples {
    topK([3,1,2], 2) => [3,2]
  }
  xs.sort_desc().take(k)
}
```

## Lexical Basics

- Identifier: `letter (letter | digit | "_")*`
- Keywords (reserved in v0.1):
  - `module`, `import`, `pub`, `type`, `if`, `else`, `for`, `while`, `repeat`, `match`, `return`, `go`, `select`
- Contract annotations begin with `@` and are not valid identifiers.

## Blocks

V0.1 supports brace blocks for declarations and control flow:

```txt
transfer(from, to, amount) {
  if amount <= 0 {
    return err("invalid amount")
  }
  ok()
}
```

Notes:

- Braces are chosen for unambiguous parsing in early compiler stages.
- Newline and indentation are non-semantic in v0.1 (formatting concern only).

## Functions

### Function Forms

- Local/internal function:
  - `name(arg1, arg2) { ... }`
- Public API function:
  - `pub name(arg1: Type, arg2: Type) -> ReturnType { ... }`

Types are optional in local code and encouraged for exported API boundaries.

### Return Rules

- `return expr` returns immediately.
- If omitted, the last expression in a block becomes the return value.

## Bindings and Assignment

- Inferred local binding: `name := expr`
- Re-assignment: `name = expr`

Examples:

```txt
acc := 0
acc = acc + 1
```

## Collections and Literals

- List: `[1, 2, 3]`
- Map: `{"a": 1, "b": 2}`
- String: `"hello"`
- Number literals: `42`, `3.14`
- Boolean: `true`, `false`

## Call and Method Chain Style

Both call styles are supported:

- Function style: `take(sorted, k)`
- Chain style: `xs.sort_desc().take(k)`

Chain style is preferred in docs for left-to-right readability.

## Error Propagation

- `expr?` propagates failure to caller.
- Explicit branch handling remains available via `match`.

Example:

```txt
data := json.parse(raw)?
validated := UserInput.validate(data)?
```

## Concurrency Syntax

- Spawn concurrent task: `go worker(jobs, out)`
- Create channels: `jobs := chan(1024)`
- Select over multiple events:

```txt
select {
  case msg := inbox.recv() => handle(msg)
  case after 5s => log.warn("idle")
  case closed inbox => break
}
```

## Contracts and Intent Annotations

Contracts can be attached at function start:

- `@intent "human-readable objective"`
- `@examples { ... }`
- `@require predicate`
- `@ensure predicate`
- `@effect effect_name`

Example:

```txt
topK(xs, k) {
  @intent "k largest numbers, sorted desc"
  @require k >= 0
  @ensure len(.) == min(k, len(xs))
  @effect alloc

  xs.sort_desc().take(k)
}
```

`.` represents the function result value in postconditions.

## Minimal Grammar Sketch (EBNF-ish)

```txt
file          := moduleDecl? importDecl* declaration*
declaration   := functionDecl | typeDecl
functionDecl  := visibility? ident "(" paramList? ")" returnType? block
block         := "{" statement* expr? "}"
statement     := binding | assignment | ifStmt | forStmt | whileStmt | selectStmt | returnStmt | contractStmt | exprStmt
binding       := ident ":=" expr
assignment    := lhs "=" expr
contractStmt  := "@intent" string
               | "@examples" "{" exampleCase* "}"
               | "@require" expr
               | "@ensure" expr
               | "@effect" ident
exampleCase   := expr "=>" expr
```

## Formatting Guidance

- Keep line length around 100 chars.
- Place contracts before executable statements.
- Prefer one transformation per line in chains when readability improves.
