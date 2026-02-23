# Chapter 11: Lexical Structure, Keywords, and Literals

This chapter is a detailed language-reference chapter for core lexical forms.
If you are building parsers, linters, generators, or writing style guides, this
chapter is especially important.

## 11.1 Lexical Overview

A VibeLang source file is tokenized into identifiers, keywords, operators,
literals, punctuation, and annotations.

Key lexical design goals:

- low-noise readability,
- deterministic parsing behavior,
- explicit signal where semantics matter (contracts, effects, concurrency forms).

The grammar source of truth is the current grammar file under `docs/spec/`.

## 11.2 Reserved Keywords

Current reserved keyword groups:

### Module and declaration

- `module`, `import`, `pub`, `type`

### Async and concurrency

- `async`, `await`, `thread`, `go`
- `select`, `after`, `closed`

### Control flow

- `if`, `else`, `for`, `while`, `repeat`, `match`, `case`, `default`
- `return`, `break`, `continue`

### Binding and constants

- `const`, `mut`

### Literal-like words

- `true`, `false`, `none`

These words are reserved and cannot be repurposed as identifiers.

## 11.3 Identifiers and Naming Guidance

While exact naming policy is a style concern rather than grammar law, practical
guidance for maintainability:

- use descriptive names at module boundaries,
- keep short names for tight local scopes only,
- avoid names that shadow domain terminology with ambiguous abbreviations,
- avoid naming that can be confused with effects or annotations.

For public APIs, prefer readability over brevity.

## 11.4 Annotation Tokens

Contract/intent/effect annotations begin with `@` and are treated distinctly
from plain identifiers:

- `@intent`
- `@examples`
- `@require`
- `@ensure`
- `@effect`

This lexical distinction keeps verification metadata visible and hard to miss in
review.

## 11.5 Integer Literals

Supported integer forms include unsuffixed and suffixed values:

- `42`
- `42i32`
- `42u64`

From numeric semantics:

- default integer literal type: `i64` unless context requires another integer
  type,
- suffixes are authoritative,
- out-of-range literals are compile-time errors.

## 11.6 Floating-Point Literals

Float examples:

- `3.14`
- `0.5f32`
- `2.0f64`

Semantic defaults:

- unsuffixed float defaults to `f64`,
- suffixes choose explicit precision.

Use explicit `f32` where memory/bandwidth profile requires it.

## 11.7 String Literals

String literals represent UTF-8 text:

```txt
"hello\nworld"
```

Supported escape baseline includes:

- `\\`
- `\"`
- `\n`, `\r`, `\t`
- `\u{...}` for Unicode scalar escapes

Invalid escapes are parse-time errors.

## 11.8 Character Literals

Character literals represent one Unicode scalar value:

```txt
'x'
```

Multi-scalar grapheme clusters are not valid character literals.

## 11.9 Boolean and Optional Literals

Boolean values:

- `true`
- `false`

Optional empty value:

- `none`

Optional type syntax uses `T?`:

- `Str?`
- `List<i64>?`

## 11.10 List and Map Literals

List literal:

```txt
[1, 2, 3]
```

Map literal:

```txt
{"a": 1, "b": 2}
```

Empty literals (`[]`, `{}`) require type context or explicit annotation to avoid
ambiguity.

## 11.11 Duration Literals

Duration literals are part of practical concurrency syntax:

- `5ms`
- `1s`
- `2m`
- `1h`

These are especially useful with `select` timeout cases (`case after 5s => ...`).

## 11.12 Punctuation and Operators (Lexical View)

Frequently seen punctuation:

- `(` `)` `{` `}` `[` `]`
- `,` `:` `.`

Frequently seen operators/forms:

- `:=` binding
- `=` assignment
- `==`, `!=`, `<`, `<=`, `>`, `>=`
- `+`, `-`, `*`, `/`, `%`
- `=>` match/select/example case mapping
- `->` function return typing
- postfix `?` result propagation

## 11.13 Lexical Ambiguity Avoidance

VibeLang emphasizes deterministic parse and diagnostics. To reduce ambiguity in
real code:

- keep contracts grouped at top of function body,
- avoid deeply nested expression tricks in one line,
- split long chains into intermediate bindings with intentful names,
- avoid mixed-style numeric literal usage without explicit rationale.

## 11.14 Readability Conventions

Practical conventions that scale:

- keep line length around ~100 chars,
- one transformation per line in long chains,
- place related contracts in logical order:
  1. `@intent`
  2. `@examples`
  3. `@require`
  4. `@ensure`
  5. `@effect`

This improves both human scanning and automated review tooling quality.

## 11.15 Reference Example Bringing It Together

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

You can inspect this one function and see lexical categories, contracts, numeric
literals, and control forms in one place.

## 11.16 Clarification: Literal and Keyword Rules Are About Predictability

This chapter may look "reference-heavy," but the purpose is practical: lexical
and literal rules are where many subtle regressions begin. Inconsistent literal
typing, ambiguous names, and poorly understood keyword semantics can propagate
into type errors, runtime surprises, and migration friction.

Treat this chapter as a stability tool. Teams that standardize these low-level
rules early usually spend less time on noisy review debates and more time on
actual product logic.

## 11.17 Chapter Checklist

You should now be able to:

- list and classify reserved keywords,
- apply literal forms correctly (numeric, string, char, container, duration),
- use optional syntax and `none` correctly,
- write lexically clear code that remains easy to parse and review.

---

Next: Chapter 12 goes deeper into expressions and control-flow semantics.
