# Chapter 8: Error Handling with Result

Every program encounters failure. Files go missing. Networks time out. Users
enter garbage. The question is not whether your program will face errors, but
how it represents, propagates, and recovers from them.

VibeLang takes a firm position: errors are values, not exceptions. There is no
`try`/`catch`. There is no `null`. There is `Result<T, E>` — a type that forces
you to acknowledge that an operation can fail and decide what to do about it.

This chapter covers VibeLang's error handling model from first principles through
advanced patterns in concurrent code.

---

## 8.1 VibeLang's Error Philosophy

### No Exceptions

Many languages use exceptions for error handling. A function throws an exception,
and some caller somewhere up the stack catches it — or does not, and the program
crashes. This model has three problems:

1. **Invisible control flow.** You cannot tell from a function's signature
   whether it might throw. Any function might throw anything at any time.
2. **Forgotten handlers.** It is easy to forget to catch an exception. The
   compiler does not remind you.
3. **Nondeterministic unwinding.** Exception unwinding interacts poorly with
   resource management, concurrency, and AOT compilation.

VibeLang eliminates all three problems by making errors part of the return type.
If a function can fail, its signature says so. If you call that function, the
compiler forces you to handle the error case.

### No Null

In many languages, `null` is the billion-dollar mistake — a value that can
inhabit any type and causes crashes when you forget to check for it. VibeLang
has no implicit null. If a value might be absent, you use `T?` (the optional
type) or `Result<T, E>` (if the absence represents an error). Both require
explicit handling.

### Errors Are Values

In VibeLang, an error is not a special control flow mechanism. It is a regular
value of type `Result<T, E>`. You can store it in a variable, pass it to a
function, put it in a list, or pattern match on it. There is nothing magical
about it.

```vibe
pub read_config(path: Str) -> Result<Config, IoError> {
    @intent "read and parse a configuration file"
    @effect io
    @effect alloc

    content := read_file(path)?
    parse_config(content)
}
```

The return type `Result<Config, IoError>` tells every caller: this function
either succeeds with a `Config` or fails with an `IoError`. The caller must
handle both cases. The compiler enforces this.

---

## 8.2 The Result Type

### Structure

`Result<T, E>` has exactly two variants:

- `ok(value)` — the operation succeeded, and `value` is the result of type `T`
- `err(error)` — the operation failed, and `error` is the error of type `E`

### Creating Results

```vibe
pub divide(a: f64, b: f64) -> Result<f64, MathError> {
    @intent "divide a by b, returning error on division by zero"

    if b == 0.0 {
        err(MathError.division_by_zero())
    } else {
        ok(a / b)
    }
}
```

The function returns `ok(a / b)` on the happy path and `err(...)` on the error
path. Both are explicit, visible, and type-checked.

### Pattern Matching on Result

The most fundamental way to handle a `Result` is pattern matching:

```vibe
pub main() -> Int {
    @effect io

    result := divide(10.0, 3.0)

    match result {
        ok(value) => {
            println("Result: " + to_str(value))
            0
        },
        err(e) => {
            println("Error: " + e.message())
            1
        },
    }
}
```

The `match` expression forces you to handle both cases. If you forget the `err`
arm, the compiler rejects the code:

```
error[E0201]: non-exhaustive match
 --> src/main.yb:8:5
  |
8 |     match result {
  |     ^^^^^ missing arm for `err(_)`
  |
  = help: add `err(e) => { ... }` to handle the error case
```

This is the core guarantee of VibeLang's error model: you cannot accidentally
ignore an error. The type system makes error handling mandatory.

### Nested Results

Sometimes operations produce nested results. For example, parsing a string that
might fail, then converting the parsed value, which might also fail:

```vibe
pub parse_positive_int(raw: Str) -> Result<i64, ParseError> {
    @intent "parse a string as a positive integer"

    n := parse_i64(raw)?
    if n <= 0 {
        err(ParseError.not_positive(n))
    } else {
        ok(n)
    }
}
```

The `?` operator (covered in the next section) keeps this flat and readable
despite multiple potential failure points.

---

## 8.3 The `?` Operator

### The Problem: Propagation Boilerplate

Without the `?` operator, propagating errors requires verbose pattern matching
at every step:

```vibe
pub process_file(path: Str) -> Result<Summary, AppError> {
    @effect io
    @effect alloc

    content_result := read_file(path)
    content := match content_result {
        ok(c) => c,
        err(e) => { return err(AppError.from_io(e)) },
    }

    parsed_result := parse_data(content)
    parsed := match parsed_result {
        ok(p) => p,
        err(e) => { return err(AppError.from_parse(e)) },
    }

    ok(summarize(parsed))
}
```

This is correct but noisy. The actual logic — read, parse, summarize — is buried
under error handling boilerplate.

### The Solution: `?`

The `?` operator does exactly what the boilerplate above does, but in a single
character:

```vibe
pub process_file(path: Str) -> Result<Summary, AppError> {
    @intent "read a file, parse its contents, and return a summary"
    @effect io
    @effect alloc

    content := read_file(path)?
    parsed := parse_data(content)?
    ok(summarize(parsed))
}
```

When applied to a `Result` value:
- If the value is `ok(v)`, `?` extracts `v` and execution continues
- If the value is `err(e)`, `?` returns early from the enclosing function with
  `err(e)`

The three-line version is semantically identical to the twelve-line version
above. The `?` operator is pure syntactic sugar for the match-and-early-return
pattern.

### Chaining Operations with `?`

The `?` operator shines when you have a sequence of fallible operations:

```vibe
pub deploy_service(config_path: Str) -> Result<DeployResult, DeployError> {
    @intent "read config, validate it, build the artifact, and deploy"
    @effect io
    @effect net
    @effect alloc

    config := read_config(config_path)?
    validated := validate_config(config)?
    artifact := build_artifact(validated)?
    result := upload_and_deploy(artifact)?
    ok(result)
}
```

Each `?` is a potential early return. If `read_config` fails, the function
returns immediately with that error. If `validate_config` fails, same thing.
The happy path reads top-to-bottom as a sequence of steps, with error
propagation handled implicitly.

### `?` Only Works in Functions Returning Result

The `?` operator can only be used inside a function whose return type is
`Result<T, E>`. Using it in a function that returns a plain value is a compiler
error:

```vibe
pub main() -> Int {
    content := read_file("config.toml")?  // Error!
    0
}
```

```
error[E0203]: `?` operator in non-Result function
 --> src/main.yb:2:42
  |
2 |     content := read_file("config.toml")?
  |                                        ^ cannot use `?` here
  |
  = note: `main` returns `Int`, not `Result<_, _>`
  = help: handle the error with `match` or change the return type to `Result`
```

This is intentional. The `?` operator propagates errors to the caller. If the
function does not return `Result`, there is nowhere to propagate to. You must
handle the error explicitly.

### Type Compatibility

The error type of the `?` expression must be compatible with the function's
error type. If `read_file` returns `Result<Str, IoError>` but your function
returns `Result<Summary, AppError>`, the `IoError` must be convertible to
`AppError`. VibeLang supports this through error conversion traits:

```vibe
type AppError {
    message: Str,
    source: Str,
}

impl From<IoError> for AppError {
    pub from(e: IoError) -> AppError {
        AppError {
            message: e.message(),
            source: "io",
        }
    }
}

impl From<ParseError> for AppError {
    pub from(e: ParseError) -> AppError {
        AppError {
            message: e.message(),
            source: "parse",
        }
    }
}
```

With these conversions defined, `?` automatically converts the inner error type
to the outer error type. If no conversion exists, the compiler tells you:

```
error[E0204]: incompatible error types
 --> src/deploy.yb:6:44
  |
6 |     config := read_config(config_path)?
  |                                       ^ `IoError` cannot convert to `DeployError`
  |
  = help: implement `From<IoError> for DeployError`
```

---

## 8.4 Creating Functions That Return Result

### When to Return Result

Return `Result` when the failure is an **expected runtime condition** that the
caller should handle:

- File not found
- Network timeout
- Invalid user input
- Resource exhausted
- Permission denied

Do **not** return `Result` for programming bugs. Use `@require` preconditions
for those (see Section 8.6).

### Designing Error Types

Good error types carry enough information for the caller to make a decision:

```vibe
type ParseError {
    message: Str,
    line: i64,
    column: i64,
}

pub ParseError.invalid_token(token: Str, line: i64, col: i64) -> ParseError {
    ParseError {
        message: "unexpected token: " + token,
        line: line,
        column: col,
    }
}

pub ParseError.unexpected_eof(line: i64) -> ParseError {
    ParseError {
        message: "unexpected end of file",
        line: line,
        column: 0,
    }
}
```

The error type includes the error message, the line number, and the column
number. A caller can use this information to display a helpful diagnostic, retry
with different input, or propagate the error upward with additional context.

### Real-World Example: Configuration Parser

```vibe
type ConfigError {
    kind: ConfigErrorKind,
    message: Str,
    key: Str,
}

type ConfigErrorKind {
    MissingKey,
    InvalidValue,
    IoFailure,
}

pub load_config(path: Str) -> Result<AppConfig, ConfigError> {
    @intent "load and validate application configuration from a TOML file"
    @effect io
    @effect alloc

    content := read_file(path).map_err(|e| {
        ConfigError {
            kind: ConfigErrorKind.IoFailure,
            message: e.message(),
            key: "",
        }
    })?

    raw := parse_toml(content)?

    host := raw.get_str("server.host").ok_or(ConfigError {
        kind: ConfigErrorKind.MissingKey,
        message: "required key not found",
        key: "server.host",
    })?

    port_str := raw.get_str("server.port").ok_or(ConfigError {
        kind: ConfigErrorKind.MissingKey,
        message: "required key not found",
        key: "server.port",
    })?

    port := parse_u16(port_str).map_err(|_| {
        ConfigError {
            kind: ConfigErrorKind.InvalidValue,
            message: "port must be a number between 1 and 65535",
            key: "server.port",
        }
    })?

    ok(AppConfig { host: host, port: port })
}
```

Each potential failure point produces a `ConfigError` with enough context for
the caller to display a useful message or take corrective action.

---

## 8.5 Handling Errors

### Pattern Matching with `match`

The most explicit way to handle errors:

```vibe
pub main() -> Int {
    @effect io

    match load_config("app.toml") {
        ok(config) => {
            println("Server: " + config.host + ":" + to_str(config.port))
            0
        },
        err(e) => {
            match e.kind {
                ConfigErrorKind.MissingKey => {
                    println("Missing config key: " + e.key)
                    println("Please add '" + e.key + "' to app.toml")
                },
                ConfigErrorKind.InvalidValue => {
                    println("Invalid value for '" + e.key + "': " + e.message)
                },
                ConfigErrorKind.IoFailure => {
                    println("Could not read config file: " + e.message)
                },
            }
            1
        },
    }
}
```

### Default Values with `unwrap_or`

When you have a sensible default for the error case:

```vibe
pub get_timeout(config: Config) -> i64 {
    @intent "return the configured timeout, defaulting to 30 seconds"

    config.get_i64("timeout").unwrap_or(30)
}
```

`unwrap_or` returns the `ok` value if present, or the provided default if the
result is `err`. The error is silently discarded, which is appropriate when the
default is genuinely acceptable.

### Default Values with `unwrap_or_else`

When computing the default is expensive or requires context:

```vibe
pub get_cache_size(config: Config) -> i64 {
    @intent "return configured cache size or compute a reasonable default"

    config.get_i64("cache_size").unwrap_or_else(|_| {
        available_memory() / 4
    })
}
```

The closure is only called if the result is `err`, avoiding unnecessary
computation on the happy path.

### Transforming Results with `map` and `map_err`

`map` transforms the success value without touching the error:

```vibe
pub read_line_count(path: Str) -> Result<i64, IoError> {
    @intent "count the number of lines in a file"
    @effect io
    @effect alloc

    read_file(path).map(|content| {
        content.split("\n").len()
    })
}
```

`map_err` transforms the error value without touching the success:

```vibe
pub load_user_config() -> Result<Config, AppError> {
    @effect io
    @effect alloc

    read_config("~/.config/app.toml").map_err(|e| {
        AppError.config_failure("Failed to load user config: " + e.message())
    })
}
```

### Combining Results with `and_then`

`and_then` chains fallible operations, similar to `?` but as a method:

```vibe
pub parse_and_validate_port(raw: Str) -> Result<u16, ParseError> {
    @intent "parse a string as a valid port number (1-65535)"

    parse_u16(raw).and_then(|port| {
        if port < 1 {
            err(ParseError.out_of_range(port))
        } else {
            ok(port)
        }
    })
}
```

### Logging and Recovery

Sometimes you want to log an error and continue with a fallback:

```vibe
pub load_with_fallback(primary: Str, fallback: Str) -> Config {
    @intent "load config from primary path, falling back to secondary"
    @effect io
    @effect alloc

    match load_config(primary) {
        ok(config) => config,
        err(e) => {
            println("Warning: " + e.message + ", using fallback")
            match load_config(fallback) {
                ok(config) => config,
                err(e2) => {
                    println("Fatal: fallback config also failed: " + e2.message)
                    default_config()
                },
            }
        },
    }
}
```

---

## 8.6 Contract Failures vs Result Errors

This is one of the most important distinctions in VibeLang. Getting it wrong
leads to confused error handling, poor diagnostics, and brittle code.

### Contract Failures Are Programming Bugs

A `@require` violation means the caller passed arguments that the function
explicitly said it cannot handle. A `@ensure` violation means the implementation
failed to deliver what it promised. Both are **bugs** — defects in the code that
should be fixed, not handled at runtime.

```vibe
pub withdraw(account: Account, amount: i64) -> Result<Account, BankError> {
    @require amount > 0              // Bug if caller passes negative amount
    @require amount <= account.balance  // Bug if caller ignores balance check

    // ...
}
```

If someone calls `withdraw(account, -50)`, the `@require amount > 0`
precondition fails. This is not a "negative amount error" that the caller should
catch and retry — it is a programming mistake. The caller should never have
passed a negative amount. The fix is to fix the caller, not to add error
handling.

### Result Errors Are Expected Runtime Conditions

A `Result` error represents something that can legitimately happen during normal
operation:

```vibe
pub withdraw(account: Account, amount: i64) -> Result<Account, BankError> {
    @require amount > 0
    @require amount <= account.balance

    result := debit_ledger(account.id, amount)
    match result {
        ok(updated) => ok(updated),
        err(e) => err(BankError.ledger_failure(e)),  // Expected: ledger might be down
    }
}
```

The ledger being unavailable is not a programming bug — it is a runtime
condition. The network might be down. The database might be overloaded. This is
a `Result` error.

### Decision Framework

Ask yourself: "If this failure happens, is it a bug in the code or an expected
condition in the environment?"

| Situation | Mechanism | Rationale |
|---|---|---|
| Caller passes negative amount to `withdraw` | `@require amount > 0` | Caller bug |
| Caller passes amount exceeding balance | `@require amount <= balance` | Caller bug |
| Database is temporarily unavailable | `Result` with `err(...)` | Expected runtime condition |
| Network request times out | `Result` with `err(...)` | Expected runtime condition |
| User enters non-numeric text for port | `Result` with `err(...)` | Expected runtime condition |
| Internal invariant violated after computation | `@ensure` | Implementation bug |

### What Happens at Runtime

Contract failures and Result errors behave completely differently:

**Contract failure (dev/test profile):**
```
CONTRACT VIOLATION — ABORTING
  function: withdraw
  file:     src/bank.yb:2
  require:  amount > 0
  actual:   amount = -50

  Stack trace:
    src/bank.yb:2    withdraw
    src/handler.yb:15 handle_withdrawal
    src/main.yb:8     main
```

The program aborts with a diagnostic. This is intentional — contract violations
are bugs, and bugs should be loud and immediate in development.

**Result error:**
```vibe
match withdraw(account, 100) {
    ok(updated) => {
        println("Withdrawal successful")
    },
    err(e) => {
        println("Withdrawal failed: " + e.message())
        // Retry, show error to user, log, etc.
    },
}
```

The program continues. The caller decides what to do. This is normal control
flow, not an abort.

### Common Mistake: Using Result for Precondition Violations

```vibe
// Bad: using Result for what should be a precondition
pub withdraw(account: Account, amount: i64) -> Result<Account, BankError> {
    if amount <= 0 {
        return err(BankError.invalid_amount())  // This is a caller bug, not a runtime error
    }
    // ...
}
```

This forces every caller to handle an "error" that should never happen if the
code is correct. Use `@require` instead — it catches the bug at the source and
produces a clear diagnostic.

### Common Mistake: Using Contracts for Expected Failures

```vibe
// Bad: using @require for what should be a Result error
pub fetch_user(id: Str) -> User {
    @require user_exists_in_database(id)  // This calls the database in a contract!
    // ...
}
```

This is wrong for two reasons: the precondition performs I/O (checking the
database), which violates the purity requirement for contract expressions, and
a missing user is an expected runtime condition, not a caller bug.

---

## 8.7 Error Handling in Concurrent Code

Concurrency introduces new challenges for error handling. When you spawn a task
with `go`, how do errors from that task reach the caller? When you send values
through channels, how do you communicate failure?

### Errors Across Channel Boundaries

The standard pattern is to send `Result` values through channels:

```vibe
pub fetch_all(urls: List<Str>) -> List<Result<Response, NetError>> {
    @intent "fetch all URLs concurrently and collect results"
    @effect concurrency
    @effect net
    @effect alloc

    ch := chan(len(urls))

    for url in urls {
        go {
            result := http_get(url)
            ch.send(result)
        }
    }

    mut results := List.new()
    mut received := 0
    for received < len(urls) {
        results.append(ch.recv())
        received = received + 1
    }
    results
}
```

Each spawned task sends its `Result` through the channel. The caller receives a
list of results and can handle successes and failures individually:

```vibe
pub main() -> Int {
    @effect io
    @effect net
    @effect concurrency
    @effect alloc

    urls := ["https://api.example.com/a", "https://api.example.com/b"]
    results := fetch_all(urls)

    for result in results {
        match result {
            ok(response) => println("OK: " + response.status_str()),
            err(e) => println("Failed: " + e.message()),
        }
    }
    0
}
```

### Error Propagation in `go` Tasks

A `go` task cannot directly return an error to its spawner — it runs
independently. The only way to communicate errors back is through channels or
shared state:

```vibe
type TaskResult {
    url: Str,
    result: Result<Str, NetError>,
}

pub fetch_with_context(urls: List<Str>) -> List<TaskResult> {
    @intent "fetch URLs concurrently, returning results with source context"
    @effect concurrency
    @effect net
    @effect alloc

    ch := chan(len(urls))

    for url in urls {
        captured_url := url
        go {
            result := http_get(captured_url)
            ch.send(TaskResult {
                url: captured_url,
                result: result.map(|r| { r.body() }),
            })
        }
    }

    mut results := List.new()
    mut received := 0
    for received < len(urls) {
        results.append(ch.recv())
        received = received + 1
    }
    results
}
```

By wrapping the result in a `TaskResult` that includes the URL, the caller knows
which request failed and can retry selectively.

### Partial Failure Patterns

In concurrent operations, some tasks may succeed while others fail. VibeLang's
`Result` type makes partial failure explicit:

```vibe
pub process_batch(items: List<Item>) -> BatchResult {
    @intent "process items concurrently, collecting successes and failures separately"
    @effect concurrency
    @effect alloc

    ch := chan(len(items))

    for item in items {
        go {
            ch.send(process_item(item))
        }
    }

    mut successes := List.new()
    mut failures := List.new()
    mut received := 0

    for received < len(items) {
        match ch.recv() {
            ok(output) => successes.append(output),
            err(e) => failures.append(e),
        }
        received = received + 1
    }

    BatchResult {
        successes: successes,
        failures: failures,
        total: len(items),
    }
}
```

The caller receives a `BatchResult` that clearly separates successes from
failures, with counts for both. This is a common pattern in data processing
pipelines where partial success is acceptable.

### Timeout Patterns

Network and I/O operations may hang indefinitely. Combine `select` with a timer
channel to implement timeouts:

```vibe
pub fetch_with_timeout(url: Str, timeout_ms: i64) -> Result<Response, FetchError> {
    @intent "fetch URL with a timeout, returning error if deadline exceeded"
    @effect concurrency
    @effect net
    @effect alloc

    result_ch := chan(1)
    timer_ch := after(timeout_ms)

    go {
        result_ch.send(http_get(url))
    }

    select {
        case result := result_ch.recv() => {
            result.map_err(|e| { FetchError.network(e) })
        },
        case _ := timer_ch.recv() => {
            err(FetchError.timeout(url, timeout_ms))
        },
    }
}
```

The `select` statement races the HTTP request against a timer. Whichever
completes first determines the result. If the timer fires first, the function
returns a timeout error. The spawned `go` task will eventually complete, but its
result is discarded.

---

## 8.8 Composing Error Handling Strategies

Real programs combine multiple error handling techniques. Here is a pattern that
ties together `Result`, `?`, `match`, and contracts:

```vibe
pub sync_user_data(user_id: Str) -> Result<SyncReport, SyncError> {
    @intent "fetch remote user data, validate it, and update local store"
    @require len(user_id) > 0
    @ensure . == ok(_) implies sync_report_is_valid(.)
    @effect net
    @effect io
    @effect alloc

    remote_data := fetch_remote_user(user_id)?

    validated := match validate_user_data(remote_data) {
        ok(v) => v,
        err(e) => {
            log_warning("Validation failed for user " + user_id + ": " + e.message())
            return err(SyncError.validation_failed(e))
        },
    }

    update_local_store(user_id, validated)?

    ok(SyncReport {
        user_id: user_id,
        fields_updated: validated.changed_fields(),
        timestamp: current_time_ms(),
    })
}
```

This function demonstrates:
- `@require` for caller bugs (empty user ID)
- `@ensure` for implementation guarantees (valid sync report)
- `?` for straightforward error propagation (`fetch_remote_user`, `update_local_store`)
- `match` for error handling with side effects (logging before returning)
- `@effect` declarations for operational transparency

---

## 8.9 Anti-Patterns

### Ignoring Errors

VibeLang's type system makes it hard to ignore errors, but not impossible:

```vibe
// Bad: discarding the Result
pub bad_save(data: Data) -> Int {
    @effect io

    _ := write_file("output.txt", serialize(data))  // Error silently discarded
    0
}
```

The `_ :=` binding discards the result. The compiler may warn about this, but
it is syntactically valid. Do not do this unless you genuinely do not care about
the failure (which is rare).

### Unwrapping Without Handling

```vibe
// Bad: crashes on error
pub risky_parse(raw: Str) -> i64 {
    parse_i64(raw).unwrap()  // Panics if raw is not a valid integer
}
```

`unwrap()` extracts the `ok` value or panics on `err`. This is occasionally
useful in tests or prototypes, but in production code it converts a recoverable
error into an unrecoverable crash. Prefer `?`, `match`, or `unwrap_or`.

### Over-Wrapping Errors

```vibe
// Bad: wrapping without adding information
pub load(path: Str) -> Result<Data, AppError> {
    @effect io
    @effect alloc

    read_file(path).map_err(|e| {
        AppError.new(e.message())  // Just copies the message — adds nothing
    })
}
```

If your error wrapper does not add context (which file? which operation? what
was the caller trying to do?), it is noise. Either propagate the original error
or add meaningful context.

### Using Result Where a Contract Belongs

```vibe
// Bad: returning Result for a programming bug
pub get_element(list: List<i64>, index: i64) -> Result<i64, IndexError> {
    if index < 0 || index >= len(list) {
        err(IndexError.out_of_bounds(index, len(list)))
    } else {
        ok(list[index])
    }
}
```

If the caller is expected to always pass a valid index, use `@require` instead.
If the index comes from untrusted input, `Result` is appropriate. The choice
depends on the function's contract with its callers.

---

## 8.10 Summary

VibeLang's error handling model is built on a simple principle: errors are values,
and the type system ensures they are handled.

- **`Result<T, E>`** represents operations that can succeed (`ok(value)`) or
  fail (`err(error)`). Pattern matching ensures both cases are handled.
- **The `?` operator** propagates errors concisely, converting verbose
  match-and-return patterns into a single character. It only works in functions
  that return `Result`.
- **Error types** should carry enough context for callers to make informed
  decisions. Include what went wrong, where, and what the caller can do about it.
- **Contract failures are not errors.** `@require` and `@ensure` violations are
  programming bugs caught by the contract system. `Result` errors are expected
  runtime conditions handled by application logic. Conflating the two leads to
  confused error handling.
- **Concurrent error handling** uses channels to communicate `Result` values
  between tasks. Patterns like partial failure collection and timeouts with
  `select` handle the complexity of concurrent operations.

Together with contracts (Chapter 6) and effects (Chapter 7), VibeLang's error
handling creates a three-layer safety system:

1. **Types** ensure data has the right shape
2. **Contracts** ensure values are in the right range and functions deliver their
   promises
3. **Result** ensures expected failures are acknowledged and handled

No single layer is sufficient. Together, they produce programs that are correct,
transparent, and maintainable — even as teams grow, codebases evolve, and AI
assistants generate code at scale.
