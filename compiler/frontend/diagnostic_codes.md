# VibeLang Frontend Diagnostic Codes (Phase 1)

This catalog reserves deterministic code ranges for frontend diagnostics.

## Code Ranges

- `E1xxx` Parse/Lex diagnostics
- `E2xxx` Binder/Type diagnostics
- `E3xxx` Contract/Effect diagnostics

## Current Codes

### Parse/Lex (`E1xxx`)

- `E1001` unexpected character during lexing
- `E1002` unterminated string literal
- `E1101` expected declaration
- `E1102` expected function name
- `E1103` expected `(` after function name
- `E1104` expected `)` after parameter list
- `E1105` expected `{` to start function body
- `E1106` expected `}` to close function body
- `E1107` expected parameter name
- `E1150` expected identifier
- `E1151` expected identifier after `.`
- `E1199` parser recovery made no progress inside function body
- `E1201` expected loop variable in `for`
- `E1202` expected `in` in `for` statement
- `E1203` expected `{` after `select`
- `E1204` expected `case` in `select` block
- `E1205` expected `=>` in `select` case
- `E1206` expected `}` to close `select`
- `E1207` expected identifier after `closed`
- `E1208` expected binding identifier in `select` receive
- `E1209` expected `:=` in receive case
- `E1299` parser recovery made no progress inside `select`
- `E1210` expected `{` to start block
- `E1211` expected `}` to close block
- `E1212` parser recovery made no progress inside block
- `E1301` expected `@`
- `E1302` expected contract annotation name
- `E1303` expected string literal after `@intent`
- `E1304` expected effect name after `@effect`
- `E1305` unknown contract annotation
- `E1306` expected `{` after `@examples`
- `E1307` expected `=>` in example case
- `E1308` expected `}` to close `@examples`
- `E1309` invalid contract position (must appear before executable statements)
- `E1399` parser recovery made no progress inside `@examples`
- `E1401` expected expression
- `E1402` expected right-hand expression
- `E1403` expected expression after unary `-`
- `E1404` expected expression after unary `!`
- `E1405` expected identifier after `.`
- `E1406` expected `)` after call arguments
- `E1407` expected `)` after `old(` expression
- `E1408` expected `]`
- `E1409` expected `)`
- `E1410` unexpected token in expression

### Binder/Type (`E2xxx`)

- `E2001` unknown identifier
- `E2002` duplicate function
- `E2003` public function parameter missing explicit type (warning)
- `E2004` public function missing explicit return type (warning)
- `E2101` assignment to unknown variable
- `E2102` assignment type mismatch
- `E2103` non-boolean `if` condition
- `E2104` non-boolean `while` condition
- `E2105` non-integer `repeat` count
- `E2106` `closed` case references unknown symbol (warning)
- `E2201` return type mismatch
- `E2202` binary operation type mismatch
- `E2203` `?` used on non-Result expression
- `E2204` `.` placeholder outside `@ensure`
- `E2205` `old(...)` outside `@ensure`
- `E2206` invalid `.` usage in contract
- `E2207` invalid `old(...)` usage in contract
- `E2301` HIR verification failure

### Contract/Effect (`E3xxx`)

- `E3001` unknown effect name
- `E3002` observed effect not declared (warning)
- `E3003` declared effect not observed (info)
- `E3004` `@require` expression should be Bool
- `E3005` `@ensure` expression should be Bool
- `E3101` transitive callee effect missing from caller declaration (warning)
- `E3201` non-sendable value passed to `go`
- `E3202` unsafe member capture in `go` argument
- `E3203` shared mutable member assignment in concurrent context without explicit synchronization
- `E3301` unsupported `go` target shape in native backend
- `E3302` unknown function target in `go` native lowering
- `E3303` unsupported `go` argument type in native backend
- `E3304` unsupported `go` argument arity in native backend
- `E3401` list/map native lowering unavailable in v0.1 backend
- `E3402` member access native lowering unavailable in v0.1 backend
- `E3403` unknown call target in native backend
- `E3404` unsupported member-call lowering in native backend
- `E3405` dynamic call target unsupported in native backend
- `E3406` unsupported binary op in native backend
- `E3407` unsupported unary op in native backend

## Deterministic Ordering

Diagnostics are sorted by:

1. `line_start`
2. `col_start`
3. `line_end`
4. `col_end`
5. diagnostic `code`
6. severity rank (`error`, `warning`, `info`)
