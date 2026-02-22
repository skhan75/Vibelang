# VibeLang Syntax Color Guide (VS Code/Cursor)

Date: 2026-02-22

## Purpose

This guide defines syntax-only color intent for VibeLang editor support.
It does not define full workbench theming. Color decisions map to TextMate
scopes so any compatible theme can render the same semantic separation.

## Design goals

- Keep code readable in both dark and light themes.
- Highlight contracts and effects as first-class language concepts.
- Preserve semantic grouping for literals, types, and flow control.
- Avoid over-coloring punctuation and delimiters.

## Token taxonomy

| Category | Examples | TextMate scopes | Styling intent |
| --- | --- | --- | --- |
| Keywords | `pub`, `async`, `type`, `return`, `if`, `for`, `while`, `match`, `select`, `go`, `thread`, `const`, `mut`, `module`, `import` | `keyword.control.vibelang`, `storage.modifier.vibelang` | High-contrast keyword color |
| Contract annotations | `@intent`, `@examples`, `@require`, `@ensure`, `@effect` | `storage.type.annotation.contract.vibelang` | Accent color (brand-forward) |
| Effects | `alloc`, `mut_state`, `io`, `net`, `concurrency`, `nondet` | `support.constant.effect.vibelang` | Secondary accent |
| Types | `Int`, `Str`, `Bool`, `Float`, user-defined type names | `storage.type.vibelang`, `entity.name.type.vibelang` | Type color distinct from keywords |
| Functions | declarations and callsites | `entity.name.function.vibelang`, `support.function.vibelang` | Function identity color |
| Variables/params | bindings and params | `variable.other.readwrite.vibelang`, `variable.parameter.vibelang` | Neutral semantic color |
| Literals | numbers, strings, chars, booleans, `none` | `constant.numeric.vibelang`, `string.quoted.double.vibelang`, `string.quoted.single.vibelang`, `constant.language.boolean.vibelang`, `constant.language.null.vibelang` | Literal-specific colors |
| Module paths | `foo.bar` in module/import context | `entity.name.namespace.vibelang` | Namespace color |
| Operators | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `&&`, `||`, `:=`, `=>`, `->` | `keyword.operator.vibelang` | Subtle operator contrast |
| Comments | `// ...` | `comment.line.double-slash.vibelang` | De-emphasized comment color |

## Brand-inspired accent mapping

The recommended palette direction (when a theme supports custom token colors):

- Contract annotations: neon-cyan leaning accent
- Effects and control hotspots: violet/purple accent
- Core keywords: bright but not fluorescent
- Diagnostics overlays should remain theme-native

## Grammar mapping source

The taxonomy is derived from:

- `docs/spec/grammar_v1_0.ebnf`
- `compiler/tests/fixtures/phase7/basic/syntax`
- `compiler/tests/fixtures/phase7/intermediate/annotations`

## Validation criteria

Syntax guide conformance is validated by:

- TextMate grammar static checks in `tooling/phase13/validate_textmate_grammar.py`
- Fixture coverage checks on `phase7` syntax and annotations corpora

