# Chapter 6: Contracts and Intent-Driven Development

Every language promises correctness. VibeLang enforces it. This chapter covers
the contract annotation system — VibeLang's most distinctive feature and the
foundation of its approach to building software that stays correct over time,
across teams, and through AI-assisted refactors.

By the end of this chapter you will understand how to use all five contract
annotations, how the compiler and runtime enforce them, and how to adopt a
contract-first development workflow that catches semantic drift before it reaches
production.

---

## 6.1 The Problem: Code That Compiles But Drifts

Consider a function written six months ago by a colleague who has since left the
team. The function compiles. The tests pass. But does it still do what it was
*meant* to do?

In most languages, the answer is "probably, but nobody is sure." Here is why.

### Comments Lie

```vibe
// Returns the top k elements sorted in descending order
pub top_k(items: List<i64>, k: i64) -> List<i64> {
    sorted := sort_ascending(items)
    sorted.take(k)
}
```

The comment says "descending order." The implementation sorts ascending and takes
the first `k` elements — returning the *smallest* values. The comment and the
code disagree, and the compiler has no opinion about which one is right.

### Tests Drift

Tests are better than comments, but they live in separate files. When someone
refactors `top_k`, they may update the implementation without updating the tests.
Or worse, they update the tests to match the new (broken) behavior because the
tests "should pass." Now the tests validate the wrong thing, and nobody notices
until a customer reports incorrect results.

### The AI Amplifier

AI code generation makes drift worse, not better. A language model asked to
"optimize this function" will happily produce code that is faster but
semantically different. Without machine-readable intent, there is no automated
way to detect that the optimization changed the function's meaning.

VibeLang's contract system exists to close this gap. Contracts are not comments.
They are not separate test files. They are executable, compiler-verified
annotations that live *inside* the function body and travel with the code
wherever it goes.

---

## 6.2 The Contract Annotation System

VibeLang provides five contract annotations:

| Annotation | Purpose | Checked |
|---|---|---|
| `@intent` | Human-readable purpose statement | By AI sidecar and `vibe lint --intent` |
| `@examples` | Executable input/output specifications | By `vibe test` |
| `@require` | Preconditions on function entry | At runtime (dev/test: hard fail) |
| `@ensure` | Postconditions before function return | At runtime (dev/test: hard fail) |
| `@effect` | Side effect declarations | By the compiler, transitively |

### Placement Rules

All contract annotations appear at the **top of the function body**, before any
executable statements. This is not a style preference — it is a language rule.
The compiler rejects annotations placed after executable code.

```vibe
pub clamp(value: i64, low: i64, high: i64) -> i64 {
    // Contracts go here, at the top
    @intent "constrain value to the range [low, high]"
    @require low <= high
    @ensure . >= low
    @ensure . <= high

    // Executable code follows
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}
```

Placing an annotation after executable code produces a compiler error:

```
error[E0301]: contract annotation after executable statement
 --> src/math.yb:9:5
  |
7 |     result := value + 1
  |     ------------------- executable statement here
8 |
9 |     @ensure . > 0
  |     ^^^^^^^^^^^^^ contract annotations must precede all executable code
  |
  = help: move this annotation above line 7
```

### Ordering Within Contracts

While the compiler accepts annotations in any order, the idiomatic convention
is:

1. `@intent`
2. `@examples`
3. `@require` (preconditions)
4. `@ensure` (postconditions)
5. `@effect`

This reads naturally: purpose, then specification, then constraints, then
operational characteristics.

---

## 6.3 `@intent`: Declaring Purpose

### Syntax

```vibe
@intent "short, concrete description of what this function achieves"
```

The `@intent` annotation takes a single string literal describing the function's
purpose. It answers the question: "If this function works correctly, what outcome
holds?"

### Good Intents vs Bad Intents

An intent should describe the *what*, not the *how*. It should be specific enough
that a reader (human or AI) can determine whether an implementation satisfies it.

**Good intents:**

```vibe
@intent "return the k largest elements sorted in descending order"
@intent "transfer amount from source account to destination account"
@intent "parse a port number from a raw string, rejecting values outside 1-65535"
@intent "compute the SHA-256 hash of the input bytes"
```

**Bad intents:**

```vibe
@intent "process the data"           // Too vague — what does "process" mean?
@intent "sort then filter then map"  // Describes implementation steps, not outcome
@intent "do the thing"               // Meaningless
@intent "helper function"            // What does it help with?
```

A useful test: could someone write a completely different implementation that
still satisfies this intent? If yes, the intent describes an outcome. If no, it
describes an implementation.

### How the AI Sidecar Uses Intents

The `vibe lint --intent` command invokes the AI sidecar to compare each
function's `@intent` string against its actual implementation. The sidecar
reports when it detects semantic drift:

```bash
$ vibe lint --intent --changed
```

```
warning[W0801]: possible intent drift in `top_k`
 --> src/ranking.yb:3:5
  |
3 |     @intent "return the k largest elements sorted in descending order"
  |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: implementation appears to return the k smallest elements
  = note: `sort_ascending` followed by `take(k)` yields the lowest values
  = help: verify that the implementation matches the stated intent
```

This is not a compiler error — it is a lint warning powered by semantic analysis.
The sidecar cannot prove correctness, but it can flag suspicious mismatches that
a human reviewer should investigate.

### Multiple Intents

Each function has exactly one `@intent`. If you find yourself wanting multiple
intents, the function is doing too many things. Split it.

```vibe
// Bad: one function, two responsibilities
pub process_order(order: Order) -> Receipt {
    @intent "validate order, charge payment, and send confirmation email"
    // ...
}

// Better: separate functions, each with a clear intent
pub validate_order(order: Order) -> Result<ValidOrder, ValidationError> {
    @intent "verify that all order fields are present and within allowed ranges"
    // ...
}

pub charge_payment(order: ValidOrder) -> Result<Receipt, PaymentError> {
    @intent "charge the payment method and return an approved receipt"
    // ...
}

pub send_confirmation(receipt: Receipt) -> Result<Unit, EmailError> {
    @intent "send order confirmation email to the customer"
    // ...
}
```

---

## 6.4 `@examples`: Executable Specifications

### Syntax

```vibe
@examples {
    function_name(arg1, arg2) => expected_result
    function_name(arg1, arg2) => expected_result
}
```

Each line inside the `@examples` block is an executable test case. The left side
is a call expression using the enclosing function's name. The right side is the
expected return value. The compiler generates real test functions from these
declarations.

### A Complete Example

```vibe
pub clamp_percent(value: i64, total: i64) -> i64 {
    @intent "compute what percentage value is of total, clamped to 0-100"
    @examples {
        clamp_percent(0, 10)   => 0
        clamp_percent(5, 10)   => 50
        clamp_percent(10, 10)  => 100
        clamp_percent(15, 10)  => 100
        clamp_percent(-3, 10)  => 0
    }
    @require total > 0
    @ensure . >= 0
    @ensure . <= 100

    raw := (value * 100) / total
    if raw < 0 {
        0
    } else if raw > 100 {
        100
    } else {
        raw
    }
}
```

### What the Compiler Generates

When you run `vibe test`, the compiler transforms each example line into a
standalone test function. Conceptually, the above examples become:

```vibe
test "clamp_percent(0, 10) => 0" {
    assert_eq(clamp_percent(0, 10), 0)
}

test "clamp_percent(5, 10) => 50" {
    assert_eq(clamp_percent(5, 10), 50)
}

test "clamp_percent(10, 10) => 100" {
    assert_eq(clamp_percent(10, 10), 100)
}

test "clamp_percent(15, 10) => 100" {
    assert_eq(clamp_percent(15, 10), 100)
}

test "clamp_percent(-3, 10) => 0" {
    assert_eq(clamp_percent(-3, 10), 0)
}
```

You do not write these test functions yourself. The compiler generates and runs
them automatically.

### Running Examples

```bash
$ vibe test
```

```
running 5 contract examples for clamp_percent ...
  clamp_percent(0, 10) => 0          ... ok
  clamp_percent(5, 10) => 50         ... ok
  clamp_percent(10, 10) => 100       ... ok
  clamp_percent(15, 10) => 100       ... ok
  clamp_percent(-3, 10) => 0         ... ok

test result: ok. 5 passed; 0 failed
```

### When an Example Fails

Suppose someone changes the implementation to use integer division differently,
and `clamp_percent(5, 10)` now returns `49` instead of `50`:

```
running 5 contract examples for clamp_percent ...
  clamp_percent(0, 10) => 0          ... ok
  clamp_percent(5, 10) => 50         ... FAILED
  clamp_percent(10, 10) => 100       ... ok
  clamp_percent(15, 10) => 100       ... ok
  clamp_percent(-3, 10) => 0         ... ok

failures:

---- clamp_percent example 2 ----
  expected: 50
  actual:   49
  source:   src/math.yb:5

test result: FAILED. 4 passed; 1 failed
```

The failure message tells you exactly which example failed, what was expected,
what was produced, and where the example is defined. This makes debugging fast
and unambiguous.

### Coverage Guidance

For public functions, aim for examples that cover:

1. **Happy path** — the common, expected input
2. **Boundary cases** — minimum values, maximum values, exact thresholds
3. **Edge cases** — zero, empty, negative, overflow-adjacent inputs

You do not need dozens of examples. Three to five well-chosen cases usually
provide high signal. The goal is not exhaustive testing — it is executable
documentation that catches the most common forms of drift.

### Examples with Complex Types

Examples can use constructor functions and literals for structured types:

```vibe
pub total_price(items: List<Item>) -> i64 {
    @intent "sum the price of all items in the list"
    @examples {
        total_price([])                                    => 0
        total_price([Item { price: 100 }])                 => 100
        total_price([Item { price: 100 }, Item { price: 250 }]) => 350
    }
    @require all(items, |item| { item.price >= 0 })
    @ensure . >= 0

    items.fold(0, |acc, item| { acc + item.price })
}
```

---

## 6.5 `@require`: Preconditions

### Syntax

```vibe
@require predicate_expression
```

A `@require` annotation declares a condition that must be true when the function
is called. If the condition is false, the function was called incorrectly — the
bug is in the *caller*, not in this function.

### What Happens When Preconditions Fail

In **dev and test profiles**, a precondition failure is a hard failure with a
diagnostic message:

```
contract violation: precondition failed
  function: transfer
  file:     src/bank.yb:14
  require:  amount > 0
  actual:   amount = -50

  This is a caller bug. The function `transfer` requires `amount > 0`,
  but it was called with amount = -50.
```

In **release profiles**, the behavior is configurable via the project's
`vibe.toml`:

```toml
[profile.release]
contract_checks = "log_and_continue"  # or "hard_fail" or "disabled"
```

The default release behavior is `hard_fail` because silent contract violations
in production are a common source of data corruption. Teams that need different
behavior can configure it explicitly, but the choice is always deliberate.

### Common Precondition Patterns

**Range checks:**

```vibe
pub withdraw(account: Account, amount: i64) -> Result<Account, BankError> {
    @require amount > 0
    @require amount <= account.balance
    // ...
}
```

**Non-empty collections:**

```vibe
pub average(values: List<f64>) -> f64 {
    @require len(values) > 0
    // ...
}
```

**Relational constraints:**

```vibe
pub slice(data: List<i64>, start: i64, end: i64) -> List<i64> {
    @require start >= 0
    @require end >= start
    @require end <= len(data)
    // ...
}
```

**Domain invariants:**

```vibe
pub apply_discount(price: i64, discount_pct: i64) -> i64 {
    @require price >= 0
    @require discount_pct >= 0
    @require discount_pct <= 100
    // ...
}
```

### Multiple Preconditions

You can have as many `@require` lines as needed. Each is checked independently.
If any one fails, the violation is reported with the specific predicate that was
false:

```vibe
pub create_range(low: i64, high: i64, step: i64) -> List<i64> {
    @require low <= high
    @require step > 0

    // ...
}
```

Calling `create_range(10, 5, 1)` produces:

```
contract violation: precondition failed
  function: create_range
  file:     src/range.yb:2
  require:  low <= high
  actual:   low = 10, high = 5
```

### Preconditions and the Type System

Preconditions complement the type system. The type system ensures you pass an
`i64` where an `i64` is expected. Preconditions ensure the *value* of that `i64`
is within the domain the function can handle. Together, they form two layers of
defense:

```
Type system:  "Is this the right kind of data?"
Precondition: "Is this data in the valid range for this operation?"
```

---

## 6.6 `@ensure`: Postconditions

### Syntax

```vibe
@ensure predicate_expression
```

A postcondition declares a property that must be true about the function's return
value. If the postcondition is false after the function body executes, the bug is
in *this function's implementation* — it promised something it did not deliver.

### The `.` Placeholder

Inside `@ensure`, the special symbol `.` refers to the return value of the
function:

```vibe
pub abs(x: i64) -> i64 {
    @intent "return the absolute value of x"
    @ensure . >= 0

    if x < 0 { -x } else { x }
}
```

Here, `@ensure . >= 0` means "the return value is always non-negative." If
someone changes the implementation to accidentally return a negative number, the
postcondition catches it.

### The `old(expr)` Function

`old(expr)` captures the value of an expression *at function entry time* so you
can compare it against the state at return time. This is essential for functions
that modify state:

```vibe
pub push(mut list: List<i64>, value: i64) -> List<i64> {
    @intent "append value to the end of the list"
    @ensure len(.) == old(len(list)) + 1

    list.append(value)
    list
}
```

The postcondition says: "after push, the list is exactly one element longer than
it was when the function was called." The `old(len(list))` expression is
evaluated at entry, before `list.append(value)` executes.

### Real-World Example: Bank Transfer

Here is a complete example showing how postconditions with `old()` verify a
financial operation:

```vibe
type Account {
    id: Str,
    mut balance: i64,
}

pub transfer(mut from: Account, mut to: Account, amount: i64) -> Bool {
    @intent "move amount from source account to destination account"
    @examples {
        transfer(Account { id: "A", balance: 1000 },
                 Account { id: "B", balance: 500 },
                 200) => true
    }
    @require amount > 0
    @require from.balance >= amount
    @require from.id != to.id
    @ensure from.balance == old(from.balance) - amount
    @ensure to.balance == old(to.balance) + amount
    @ensure from.balance + to.balance == old(from.balance) + old(to.balance)

    from.balance = from.balance - amount
    to.balance = to.balance + amount
    true
}
```

The three postconditions together express a conservation law: money is neither
created nor destroyed. The first two verify individual balances. The third
verifies the total is preserved. If any implementation bug causes a rounding
error, double-deduction, or missed credit, the postconditions catch it.

### Postcondition Failure Output

If the implementation has a bug — say, it deducts from `from` but forgets to
credit `to`:

```vibe
// Buggy implementation
from.balance = from.balance - amount
// Oops: forgot to.balance = to.balance + amount
true
```

The runtime produces:

```
contract violation: postcondition failed
  function: transfer
  file:     src/bank.yb:10
  ensure:   to.balance == old(to.balance) + amount
  actual:   to.balance = 500, old(to.balance) = 500, amount = 200
  expected: to.balance = 700

  The function `transfer` promised that the destination balance would
  increase by the transfer amount, but it did not.
```

This diagnostic tells you exactly what went wrong, what the values were, and
which postcondition was violated. Compare this to a traditional test failure that
says "expected 700, got 500" — the contract failure includes the *semantic
context* of why 700 was expected.

### Postconditions on Collection Operations

Postconditions are particularly valuable for functions that transform
collections, where it is easy to accidentally change the size or ordering:

```vibe
pub sort_descending(items: List<i64>) -> List<i64> {
    @intent "return items sorted from largest to smallest"
    @ensure len(.) == old(len(items))
    @ensure all_pairs(., |a, b| { a >= b })

    // implementation
    items.sort(|a, b| { b - a })
}

pub deduplicate(items: List<i64>) -> List<i64> {
    @intent "remove duplicate values, preserving first occurrence order"
    @ensure len(.) <= old(len(items))
    @ensure all_unique(.)

    // implementation
    seen := Map.new()
    result := List.new()
    for item in items {
        if !seen.contains(item) {
            seen.insert(item, true)
            result.append(item)
        }
    }
    result
}
```

---

## 6.7 Putting It All Together

Here is a complete function using all five contract annotations. We will walk
through exactly what the compiler and runtime do with each one.

```vibe
pub top_k(items: List<i64>, k: i64) -> List<i64> {
    @intent "return the k largest elements sorted in descending order"
    @examples {
        top_k([], 0)              => []
        top_k([3, 1, 4, 1, 5], 0) => []
        top_k([3, 1, 4, 1, 5], 3) => [5, 4, 3]
        top_k([3, 1, 4, 1, 5], 5) => [5, 4, 3, 1, 1]
        top_k([7], 1)             => [7]
    }
    @require k >= 0
    @require k <= len(items)
    @ensure len(.) == k
    @ensure all_pairs(., |a, b| { a >= b })
    @effect alloc

    sorted := items.sort(|a, b| { b - a })
    sorted.take(k)
}
```

### What the Compiler Does

**Step 1: Parse and validate annotation placement.** The compiler verifies that
all five annotations appear before any executable statement. If `sorted := ...`
appeared before `@effect alloc`, the compiler would reject the file.

**Step 2: Register the intent.** The `@intent` string is stored in the module's
metadata. The `vibe lint --intent` command and the AI sidecar use this metadata
to check for semantic drift between the intent and the implementation.

**Step 3: Generate test cases from examples.** Each line in `@examples` becomes
a test function. These are compiled and included in the test binary produced by
`vibe test`.

**Step 4: Inject precondition checks.** The compiler inserts runtime checks for
`@require k >= 0` and `@require k <= len(items)` at the function's entry point.
In dev/test profiles, a violation aborts with a diagnostic. In release, the
behavior follows `vibe.toml` policy.

**Step 5: Capture `old()` snapshots.** The compiler identifies any `old()`
expressions in postconditions. Here there are none, but if there were, the
compiler would insert snapshot code at function entry.

**Step 6: Inject postcondition checks.** Before the function returns, the
compiler inserts checks for `@ensure len(.) == k` and
`@ensure all_pairs(., |a, b| { a >= b })`. The `.` symbol is bound to the
actual return value.

**Step 7: Verify effect declarations.** The compiler checks that the function
body's operations are consistent with `@effect alloc`. Since `items.sort()`
allocates a new sorted list, the `alloc` effect is required. If the function
called `println()` without declaring `@effect io`, the compiler would reject it.

### Full Test Output

```bash
$ vibe test
```

```
running 5 contract examples for top_k ...
  top_k([], 0) => []                          ... ok
  top_k([3, 1, 4, 1, 5], 0) => []            ... ok
  top_k([3, 1, 4, 1, 5], 3) => [5, 4, 3]     ... ok
  top_k([3, 1, 4, 1, 5], 5) => [5, 4, 3, 1, 1] ... ok
  top_k([7], 1) => [7]                        ... ok

running 2 precondition checks for top_k ...
  require k >= 0                              ... verified
  require k <= len(items)                     ... verified

running 2 postcondition checks for top_k ...
  ensure len(.) == k                          ... verified
  ensure all_pairs(., |a, b| { a >= b })      ... verified

running effect analysis for top_k ...
  declared: alloc                             ... consistent

test result: ok. 5 examples passed; 4 contracts verified; effects consistent
```

---

## 6.8 Contract-First Development Workflow

The most effective way to use contracts is to write them *before* the
implementation. This is not test-driven development — it is contract-driven
development, and it works at a higher level of abstraction.

### Step 1: Define the Contract

Start by writing the function signature and contracts with no implementation:

```vibe
pub fibonacci(n: i64) -> i64 {
    @intent "return the nth Fibonacci number (0-indexed, starting 0, 1, 1, 2, ...)"
    @examples {
        fibonacci(0) => 0
        fibonacci(1) => 1
        fibonacci(2) => 1
        fibonacci(5) => 5
        fibonacci(10) => 55
    }
    @require n >= 0
    @ensure . >= 0

    // TODO: implement
    0
}
```

At this point, `vibe test` will fail on most examples, but the contract is
already a precise specification. You know exactly what the function should do
before writing a single line of logic.

### Step 2: Implement to Satisfy the Contract

Now write the implementation:

```vibe
pub fibonacci(n: i64) -> i64 {
    @intent "return the nth Fibonacci number (0-indexed, starting 0, 1, 1, 2, ...)"
    @examples {
        fibonacci(0) => 0
        fibonacci(1) => 1
        fibonacci(2) => 1
        fibonacci(5) => 5
        fibonacci(10) => 55
    }
    @require n >= 0
    @ensure . >= 0

    if n <= 1 {
        n
    } else {
        mut a := 0
        mut b := 1
        mut i := 2
        for i <= n {
            temp := a + b
            a = b
            b = temp
            i = i + 1
        }
        b
    }
}
```

Run `vibe test`. All examples pass. The postcondition holds. The implementation
satisfies the specification.

### Step 3: Refactor with Confidence

Six months later, someone wants to optimize `fibonacci` using memoization. The
contracts do not change — they describe *what* the function does, not *how*:

```vibe
pub fibonacci(n: i64) -> i64 {
    @intent "return the nth Fibonacci number (0-indexed, starting 0, 1, 1, 2, ...)"
    @examples {
        fibonacci(0) => 0
        fibonacci(1) => 1
        fibonacci(2) => 1
        fibonacci(5) => 5
        fibonacci(10) => 55
    }
    @require n >= 0
    @ensure . >= 0
    @effect alloc

    cache := Map.new()
    fib_memo(n, mut cache)
}
```

The examples still pass. The postcondition still holds. The new `@effect alloc`
correctly reflects that the memoized version allocates heap memory. The contract
survived the refactor intact, and `vibe test` confirms the new implementation is
semantically equivalent to the old one.

### How Contracts Survive AI-Generated Refactors

When an AI assistant refactors a function, it can change the implementation
freely but cannot remove or weaken the contracts without triggering lint
warnings. The workflow is:

1. AI generates a new implementation
2. `vibe test` runs the contract examples against the new code
3. `vibe lint --intent` checks the new code against the `@intent`
4. If anything fails, the refactor is rejected automatically

This creates a safety net that is impossible with comments alone and difficult
to achieve with separate test files that the AI might also modify.

---

## 6.9 `@effect`: Declaring Side Effects

The `@effect` annotation declares what kinds of side effects a function may
perform. While Chapter 7 covers the effects system in full depth, understanding
`@effect` as part of the contract system is essential here.

```vibe
pub save_report(report: Report) -> Result<Unit, IoError> {
    @intent "write report to disk as a JSON file"
    @require len(report.title) > 0
    @effect io
    @effect alloc

    json := serialize_json(report)
    write_file(report.path, json)?
    ok(unit)
}
```

The `@effect` annotations tell readers and tooling: this function performs I/O
and allocates memory. A pure function that calls `save_report` must also declare
those effects — the compiler enforces this transitively.

Effects are contracts about *operational behavior*. They answer the question:
"What does this function do to the outside world?"

---

## 6.10 Anti-Patterns and Best Practices

### Anti-Pattern: Over-Contracting

Adding contracts to every trivial helper creates noise without signal:

```vibe
// Over-contracted: the contracts add nothing the types don't already say
pub add(a: i64, b: i64) -> i64 {
    @intent "add two integers"
    @examples {
        add(1, 2) => 3
    }
    @ensure . == a + b

    a + b
}
```

The postcondition `@ensure . == a + b` is literally the implementation. The
intent restates the function name. The example is trivial. None of this helps
anyone. Save contracts for functions where the relationship between inputs and
outputs is non-obvious.

### Anti-Pattern: Vague Intents

```vibe
pub process(data: Data) -> Result<Output, Error> {
    @intent "process the data"
    // ...
}
```

This intent is useless. It does not help a reviewer, an AI sidecar, or a future
maintainer understand what "process" means. Be specific about the outcome.

### Anti-Pattern: Redundant Postconditions

```vibe
pub get_name(user: User) -> Str {
    @ensure . == user.name  // This is just the implementation

    user.name
}
```

If the postcondition is identical to the implementation, it provides no
additional safety. Postconditions should express properties that could be
violated by a *different* implementation — invariants that transcend any
particular way of computing the result.

### Anti-Pattern: Testing Implementation Details in Examples

```vibe
pub sort(items: List<i64>) -> List<i64> {
    @examples {
        // Bad: tests an intermediate state, not the final result
        sort([3, 1, 2]) => [1, 2, 3]  // This is fine
    }
    // ...
}
```

Examples should test observable behavior, not internal steps. If your sort
function uses quicksort vs mergesort, the examples should not care — they should
only verify the output is sorted.

### Best Practice: Contract Density by Function Importance

Not every function needs the same level of contracting:

| Function type | Recommended contracts |
|---|---|
| Public API entry points | `@intent`, `@examples`, `@require`, `@ensure`, `@effect` |
| Internal business logic | `@intent`, key `@require`/`@ensure` |
| Simple utility helpers | Maybe `@intent` only, or none |
| Trivial getters/setters | None |

### Best Practice: Use Contracts to Document Invariants

The most valuable contracts express invariants that are not obvious from the
code:

```vibe
pub rebalance(portfolio: Portfolio) -> Portfolio {
    @intent "redistribute holdings to match target allocation percentages"
    @ensure total_value(.) == old(total_value(portfolio))
    @ensure all_allocations_within_tolerance(., portfolio.targets, 0.01)

    // Complex rebalancing logic...
}
```

The postconditions here express two critical invariants: total portfolio value is
preserved (no money created or destroyed), and all allocations are within 1% of
their targets. These are properties that a code reviewer might miss in a complex
implementation but can verify instantly from the contracts.

### When NOT to Use Contracts

- **Prototyping:** When you are exploring an idea and the interface is changing
  rapidly, contracts add friction. Add them when the interface stabilizes.
- **Trivial functions:** A one-line getter does not need five annotations.
- **Performance-critical inner loops:** Contract checks have runtime cost. In
  tight inner loops where every nanosecond matters, consider whether the checks
  are worth it. (Preconditions and postconditions can be disabled in release
  profiles for these cases.)

---

## 6.11 Contracts and the Compiler Pipeline

Understanding where contracts fit in the compilation pipeline helps you reason
about their behavior:

```
  Source Code
        |
        v
  [ Parse Annotations ]    extracts @intent, @examples,
        |                  @require, @ensure, @effect
        v
  [ Type Check ]           verifies contract expressions
        |                  are well-typed
        v
  [ Effect Analysis ]      walks call graph to verify
        |                  @effect declarations
        v
  [ Code Generation ]      injects @require/@ensure checks,
        |                  generates tests from @examples
        v
  Binary + Test Binary
```

Contract expressions are type-checked like any other expression. If you write
`@require amount > "zero"`, the compiler rejects it because you cannot compare
an `i64` to a `Str`:

```
error[E0102]: type mismatch in contract expression
 --> src/bank.yb:3:14
  |
3 |     @require amount > "zero"
  |              ^^^^^^^^^^^^^^^ cannot compare `i64` with `Str`
  |
  = help: did you mean `amount > 0`?
```

---

## 6.12 Contracts in Practice: A Real-World Walkthrough

Let us build a small but realistic module — a rate limiter — using contract-first
development.

### Step 1: Define the Interface

```vibe
type RateLimiter {
    max_requests: i64,
    window_ms: i64,
    mut requests: List<i64>,
}

pub new_limiter(max_requests: i64, window_ms: i64) -> RateLimiter {
    @intent "create a rate limiter allowing max_requests per window_ms milliseconds"
    @require max_requests > 0
    @require window_ms > 0
    @ensure .max_requests == max_requests
    @ensure .window_ms == window_ms
    @ensure len(.requests) == 0
    @effect alloc

    RateLimiter {
        max_requests: max_requests,
        window_ms: window_ms,
        requests: List.new(),
    }
}
```

### Step 2: Add the Core Operation

```vibe
pub allow_request(mut limiter: RateLimiter, now_ms: i64) -> Bool {
    @intent "return true if a request at time now_ms is within the rate limit"
    @require now_ms >= 0
    @effect mut_state
    @effect alloc

    cutoff := now_ms - limiter.window_ms
    limiter.requests = limiter.requests.filter(|t| { t > cutoff })

    if len(limiter.requests) < limiter.max_requests {
        limiter.requests.append(now_ms)
        true
    } else {
        false
    }
}
```

### Step 3: Add Examples That Tell a Story

```vibe
pub allow_request(mut limiter: RateLimiter, now_ms: i64) -> Bool {
    @intent "return true if a request at time now_ms is within the rate limit"
    @examples {
        // Fresh limiter allows first request
        allow_request(new_limiter(2, 1000), 100) => true
    }
    @require now_ms >= 0
    @effect mut_state
    @effect alloc

    cutoff := now_ms - limiter.window_ms
    limiter.requests = limiter.requests.filter(|t| { t > cutoff })

    if len(limiter.requests) < limiter.max_requests {
        limiter.requests.append(now_ms)
        true
    } else {
        false
    }
}
```

### Step 4: Verify

```bash
$ vibe test
```

```
running 1 contract example for allow_request ...
  allow_request(new_limiter(2, 1000), 100) => true  ... ok

test result: ok. 1 passed; 0 failed
```

The rate limiter works, and its contracts document both its interface and its
behavioral guarantees. When someone modifies the windowing logic six months from
now, the contracts will catch any semantic drift.

---

## 6.13 Contract Failures vs Result Errors

This distinction is critical and often confusing for newcomers.

**Contract failures** (`@require` and `@ensure` violations) indicate programming
bugs. A precondition failure means the caller passed invalid arguments. A
postcondition failure means the implementation is broken. These are not expected
runtime conditions — they are defects.

**Result errors** (`err(...)`) indicate expected runtime conditions. A file might
not exist. A network request might time out. A user might enter invalid input.
These are not bugs — they are part of normal operation.

```vibe
pub parse_port(raw: Str) -> Result<u16, ParseError> {
    @intent "parse a port number from a string, returning error for invalid input"
    @require len(raw) > 0  // Caller bug if they pass empty string
    @ensure . == ok(_) implies port_value(.) >= 1
    @ensure . == ok(_) implies port_value(.) <= 65535

    n := parse_u16(raw)?
    if n < 1 || n > 65535 {
        err(ParseError.out_of_range(n))  // Expected runtime condition
    } else {
        ok(n)
    }
}
```

The `@require` catches a programming mistake (passing an empty string). The
`Result` handles a runtime condition (user typed "99999" which is out of range).
These are fundamentally different failure modes and should not be conflated.

---

## 6.14 Summary

VibeLang's contract system transforms functions from opaque code blocks into
self-documenting, self-verifying units of behavior:

- **`@intent`** declares what the function achieves, enabling AI-powered drift
  detection via `vibe lint --intent`.
- **`@examples`** provide executable specifications that the compiler turns into
  real test cases, run automatically by `vibe test`.
- **`@require`** guards function entry with preconditions, catching caller bugs
  before they propagate.
- **`@ensure`** verifies function output with postconditions, catching
  implementation bugs before they escape. The `.` placeholder and `old()`
  function enable expressive invariants.
- **`@effect`** declares operational side effects, enforced transitively by the
  compiler.

Together, these annotations create a system where intent, specification, and
verification live alongside the code they describe. They cannot drift
independently. They are checked automatically. And they provide a safety net that
survives refactors, team changes, and AI-assisted code generation.

The next chapter explores the effects system in depth — how VibeLang tracks,
enforces, and leverages side effect declarations across your entire program.
