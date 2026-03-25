# Chapter 9: Strings, Lists, and Maps

Every useful program works with data, and most data lives in one of three
containers: text, ordered sequences, or key-value associations. VibeLang ships
three built-in collection types that cover these needs: `Str`, `List<T>`, and
`Map<K,V>`. This chapter explores each in depth — their internal
representation, their APIs, the contracts they uphold, and the trade-offs you
should understand before choosing one over another.

## 9.1 Strings (`Str`)

### 9.1.1 What a String Actually Is

A `Str` in VibeLang is an immutable, heap-allocated sequence of UTF-8 encoded
bytes. That single sentence carries several consequences:

- **Immutable.** Once created, the bytes inside a `Str` never change. Operations
  like concatenation produce a *new* `Str` rather than modifying the original.
- **Heap-allocated.** Strings participate in the garbage collector's lifecycle.
  Creating strings triggers the `alloc` effect (covered in Chapter 7).
- **UTF-8.** Every `Str` is guaranteed to contain valid UTF-8. The compiler and
  runtime reject byte sequences that violate UTF-8 encoding rules.

Why UTF-8? Because it is the dominant encoding on the modern web, in file
systems, and across network protocols. By committing to a single encoding,
VibeLang eliminates an entire class of "which encoding is this?" bugs.

### 9.1.2 String Literals and Escape Sequences

String literals are delimited by double quotes:

```vibe
greeting := "Hello, world!"
empty := ""
```

VibeLang supports the following escape sequences inside string literals:

| Escape   | Meaning                        |
|----------|--------------------------------|
| `\\`     | Literal backslash              |
| `\"`     | Literal double quote           |
| `\n`     | Newline (U+000A)               |
| `\r`     | Carriage return (U+000D)       |
| `\t`     | Horizontal tab (U+0009)        |
| `\0`     | Null byte (U+0000)             |
| `\u{N}`  | Unicode scalar value in hex    |

The `\u{N}` form accepts 1–6 hex digits, matching any valid Unicode scalar
value:

```vibe
heart := "\u{2764}"
smiley := "\u{1F600}"
```

An invalid escape is a *compile-time* error, not a runtime surprise:

```vibe
bad := "hello\q"
```

```
error[E0201]: unknown escape sequence `\q`
 --> greeting.yb:1:15
  |
1 | bad := "hello\q"
  |               ^^ unknown escape
  |
  = help: valid escapes are: \\, \", \n, \r, \t, \0, \u{N}
```

### 9.1.3 Concatenation with `+`

The `+` operator joins two strings, producing a new `Str`:

```vibe
first := "Vibe"
second := "Lang"
combined := first + second
```

After this code, `combined` holds `"VibeLang"`. Neither `first` nor `second` is
modified — they remain available and unchanged.

You can chain concatenation:

```vibe
pub greet(name: Str) -> Str {
    "Hello, " + name + "!"
}
```

Because each `+` allocates a new string, building a string from many small
pieces inside a loop can be expensive. For heavy string construction, prefer
collecting parts into a `List<Str>` and joining them at the end:

```vibe
pub build_csv_row(fields: List<Str>) -> Str {
    @effect alloc

    mut result := ""
    mut i := 0
    for field in fields {
        if i > 0 {
            result = result + ","
        }
        result = result + field
        i = i + 1
    }
    result
}
```

### 9.1.4 String Length — Bytes, Not Characters

The `.len()` method on `Str` returns the number of **bytes**, not the number of
visible characters or Unicode code points:

```vibe
pub main() -> Int {
    ascii := "hello"
    emoji := "\u{1F600}"

    ascii_len := ascii.len()
    emoji_len := emoji.len()

    0
}
```

Here `ascii_len` is `5` (one byte per ASCII character), but `emoji_len` is `4`
because the grinning face emoji requires four bytes in UTF-8.

This design is intentional. Byte length is O(1) and unambiguous. "Character
count" is surprisingly complex in Unicode — a single visible glyph can span
multiple code points (combining characters, emoji sequences). VibeLang makes the
cheap, precise operation the default and provides higher-level text utilities in
the standard library for code-point or grapheme-cluster counting when you need
them.

### 9.1.5 String Slicing and Substrings

You can extract a substring using byte-offset slicing:

```vibe
text := "Hello, VibeLang!"
sub := text.slice(7, 15)
```

`sub` is `"VibeLang"`. The `slice(start, end)` method takes byte offsets and
returns a new `Str` containing bytes `[start, end)`.

**Critical rule:** both `start` and `end` must fall on valid UTF-8 character
boundaries. If they don't, the runtime produces a deterministic error rather
than returning garbled text:

```vibe
multibyte := "café"
bad_slice := multibyte.slice(3, 4)
```

The `é` character is encoded as two bytes (`0xC3 0xA9`). Slicing at byte 3 cuts
into the middle of that character, triggering:

```
runtime error: string slice index 3 is not a UTF-8 character boundary
 --> slice_demo.yb:2:15
```

This strictness prevents an entire category of mojibake bugs. When working with
text that may contain multi-byte characters, derive safe byte indices with
`std.text` helpers such as `text.index_of` and `text.split_part`, or restrict
slicing to boundaries you know are ASCII-safe.

### 9.1.6 Common String Operations

Common text helpers live in `std.text` (import `std.text`). They take the
string as an argument and return a new value — `Str` values are never modified
in place.

```vibe
import std.text

s := "  Hello, World!  "

trimmed := text.trim(s)
upper := text.to_upper(s)
lower := text.to_lower(s)
found := text.contains(s, "World")
idx := text.index_of(s, "World")
first_field := text.split_part("a,b,c", ",", 0)
starts := text.starts_with(s, "  Hello")
ends := text.ends_with(s, "!  ")
```

`text.index_of(haystack, needle)` returns the starting **byte** index of the
first occurrence of `needle`, or `-1` if it is not found. `text.split_part(s,
delimiter, n)` returns the `n`th segment when splitting on `delimiter` (0-based).

A practical example — parsing a simple key-value configuration line:

```vibe
import std.text

pub parse_config_line(line: Str) -> Result<(Str, Str), Str> {
    idx := text.index_of(line, "=")
    if idx < 0 {
        Err("missing '=' delimiter")
    } else {
        key := line.slice(0, idx).trim()
        value := line.slice(idx + 1, line.len()).trim()
        Ok((key, value))
    }
}
```

### 9.1.7 Strings in Contracts

Because strings are immutable and their length is always available, they work
naturally in contract annotations:

```vibe
pub sanitize_username(raw: Str) -> Str {
    @require raw.len() > 0, "username must not be empty"
    @require raw.len() <= 64, "username too long"
    @ensure .len() > 0
    @ensure .len() <= 64

    raw.trim().to_lower()
}
```

The `@require` preconditions validate input at the call boundary. The `@ensure`
postconditions document guarantees about the return value (`.` refers to the
return value in postconditions). If the trimmed, lowercased result somehow
violated these bounds, the contract system would catch it.

Contracts on string-returning functions are especially valuable for functions
that sanitize, format, or transform text — they make the transformation's
invariants explicit and machine-checkable.

## 9.2 Lists (`List<T>`)

### 9.2.1 Creating Lists

A list is an ordered, growable sequence of elements. Create one with literal
syntax:

```vibe
numbers := [1, 2, 3, 4, 5]
names := ["Alice", "Bob", "Carol"]
empty_ints : List<Int> := []
```

The compiler infers the element type from the literal contents. For an empty
list, you must provide a type annotation so the compiler knows what `T` is.

In VibeLang v1, `List<Int>` is the primary supported list type. This constraint
keeps the compiler and runtime simple while covering the most common use cases.

### 9.2.2 Appending Elements

To add an element to a list, the list binding must be mutable:

```vibe
mut scores := [90, 85, 92]
scores.append(88)
scores.append(95)
```

After these calls, `scores` contains `[90, 85, 92, 88, 95]`.

Attempting to append to an immutable list is a compile-time error:

```vibe
scores := [90, 85, 92]
scores.append(88)
```

```
error[E0305]: cannot call mutating method `append` on immutable binding `scores`
 --> grades.yb:2:1
  |
1 | scores := [90, 85, 92]
  |         -- binding is immutable
2 | scores.append(88)
  | ^^^^^^^^^^^^^^^^^ `append` requires a mutable binding
  |
  = help: change the binding to `mut scores := [90, 85, 92]`
```

This error is one of VibeLang's most common early stumbling blocks. The fix is
always the same: add `mut` to the binding.

### 9.2.3 Accessing Elements

Use `.get(index)` to retrieve an element by zero-based index:

```vibe
items := [10, 20, 30]
first := items.get(0)
second := items.get(1)
```

Out-of-bounds access produces a deterministic runtime error:

```vibe
items := [10, 20, 30]
bad := items.get(5)
```

```
runtime error: list index 5 out of bounds (length 3)
 --> access.yb:2:8
```

To update an element in place, use `.set(index, value)` on a mutable list:

```vibe
mut items := [10, 20, 30]
items.set(0, 99)
```

Now `items` is `[99, 20, 30]`.

### 9.2.4 List Length

The `.len()` method returns the number of elements:

```vibe
items := [10, 20, 30]
count := items.len()
```

`count` is `3`. This is an O(1) operation — the list tracks its length
internally.

### 9.2.5 Iterating with `for`

The `for ... in` loop is the idiomatic way to process every element:

```vibe
pub sum(numbers: List<Int>) -> Int {
    mut total := 0
    for n in numbers {
        total = total + n
    }
    total
}
```

The loop variable `n` is immutable by default. Each iteration binds `n` to the
next element in order. You cannot modify the list while iterating over it — the
compiler prevents mutation of a list that is currently being iterated.

If you need the index as well, use `enumerate`:

```vibe
pub print_ranked(names: List<Str>) -> Int {
    for (i, name) in names.enumerate() {
        print(i.to_str() + ". " + name)
    }
    0
}
```

### 9.2.6 Sorting with `.sort_desc()`

The `.sort_desc()` method sorts a mutable list in descending order, in place:

```vibe
mut scores := [72, 95, 88, 61, 90]
scores.sort_desc()
```

After sorting, `scores` is `[95, 90, 88, 72, 61]`.

Sorting requires a mutable binding because it rearranges the list's contents.
The sort is stable — elements that compare equal retain their original relative
order.

### 9.2.7 Taking Elements with `.take(n)`

The `.take(n)` method returns a new list containing the first `n` elements:

```vibe
all_scores := [95, 90, 88, 72, 61]
top_three := all_scores.take(3)
```

`top_three` is `[95, 90, 88]`. The original list is unchanged.

If `n` exceeds the list length, `.take(n)` returns a copy of the entire list
without error. This makes it safe to use without a bounds check:

```vibe
short := [1, 2]
result := short.take(100)
```

`result` is `[1, 2]`.

A common pattern combines sorting and taking to find top-N values:

```vibe
pub top_scores(scores: List<Int>, n: Int) -> List<Int> {
    @effect alloc
    @require n > 0, "n must be positive"
    @ensure .len() <= n

    mut sorted := scores
    sorted.sort_desc()
    sorted.take(n)
}
```

### 9.2.8 Lists and the `alloc` Effect

Creating a list, appending to it, or calling methods that produce new lists
(like `.take()`) all allocate memory. Functions that work with lists should
declare the `alloc` effect:

```vibe
pub double_all(numbers: List<Int>) -> List<Int> {
    @effect alloc

    mut result : List<Int> := []
    for n in numbers {
        result.append(n * 2)
    }
    result
}
```

The `@effect alloc` annotation tells callers and the compiler that this function
allocates heap memory. This is not just documentation — the effect system tracks
allocation to enable reasoning about performance-sensitive code paths.

### 9.2.9 Common List Patterns

**Filtering:**

```vibe
pub passing_scores(scores: List<Int>) -> List<Int> {
    @effect alloc
    @ensure .len() <= scores.len()

    mut result : List<Int> := []
    for score in scores {
        if score >= 70 {
            result.append(score)
        }
    }
    result
}
```

**Accumulating:**

```vibe
pub max_value(numbers: List<Int>) -> Int {
    @require numbers.len() > 0, "list must not be empty"

    mut best := numbers.get(0)
    for n in numbers {
        if n > best {
            best = n
        }
    }
    best
}
```

**Transforming into a different structure:**

```vibe
pub frequency_count(values: List<Int>) -> Map<Int, Int> {
    @effect alloc

    mut counts : Map<Int, Int> := {}
    for v in values {
        if counts.contains(v) {
            counts.set(v, counts.get(v) + 1)
        } else {
            counts.set(v, 1)
        }
    }
    counts
}
```

## 9.3 Maps (`Map<K,V>`)

### 9.3.1 Creating Maps

A map is an associative collection that stores key-value pairs. Create one with
literal syntax using curly braces:

```vibe
ages := {"Alice": 30, "Bob": 25, "Carol": 28}
config := {"timeout": 5000, "retries": 3}
empty_map : Map<Str, Int> := {}
```

Keys and values are separated by `:`, pairs are separated by `,`. As with
lists, empty maps need a type annotation.

### 9.3.2 Supported Type Combinations (v1)

VibeLang v1 supports these map type combinations:

| Key Type | Value Type | Example                              |
|----------|------------|--------------------------------------|
| `Str`    | `Int`      | `{"timeout": 5000}`                  |
| `Str`    | `Str`      | `{"name": "Ada", "role": "dev"}`     |
| `Int`    | `Int`      | `{1: 100, 2: 200}`                   |
| `Map<Str, Int>` | — | Most common for configuration data   |
| `Map<Str, Str>` | — | String metadata, headers, string props |
| `Map<Int, Int>` | — | Common for counters and histograms   |

When you need **JSON object text** from runtime-driven data, prefer
`std.json` **`json.builder`** (or `Json` values plus `json.stringify`) so structure
and types stay explicit. The stdlib helper **`json.from_map(Map<Str, Str>)`**
remains a **convenience** for maps that are already string-to-string (with
coercion heuristics on values); it is not the canonical JSON API—see Appendix C.6.

This constraint is deliberate. By limiting key-value combinations in v1,
VibeLang can optimize the underlying hash map implementation and provide
stronger determinism guarantees. Additional combinations may appear in future
versions.

### 9.3.3 Getting Values

Use `.get(key)` to retrieve a value:

```vibe
ages := {"Alice": 30, "Bob": 25}
alice_age := ages.get("Alice")
```

`alice_age` is `30`.

Accessing a key that doesn't exist produces a deterministic runtime error:

```vibe
ages := {"Alice": 30, "Bob": 25}
unknown := ages.get("Dave")
```

```
runtime error: key "Dave" not found in map
 --> lookup.yb:2:12
```

To avoid this, check for the key first:

```vibe
if ages.contains("Dave") {
    dave_age := ages.get("Dave")
} else {
    print("Dave not found")
}
```

### 9.3.4 Setting Values

Use `.set(key, value)` on a mutable map to insert or update a key:

```vibe
mut ages := {"Alice": 30, "Bob": 25}
ages.set("Carol", 28)
ages.set("Alice", 31)
```

After these calls, the map contains `{"Alice": 31, "Bob": 25, "Carol": 28}`.
Setting an existing key overwrites its value.

### 9.3.5 Checking for Keys

The `.contains(key)` method returns a `Bool`:

```vibe
ages := {"Alice": 30, "Bob": 25}
has_alice := ages.contains("Alice")
has_dave := ages.contains("Dave")
```

`has_alice` is `true`, `has_dave` is `false`.

### 9.3.6 Removing Entries

Use `.remove(key)` on a mutable map:

```vibe
mut ages := {"Alice": 30, "Bob": 25, "Carol": 28}
ages.remove("Bob")
```

The map now contains `{"Alice": 30, "Carol": 28}`. Removing a key that doesn't
exist is a no-op — it does not produce an error.

### 9.3.7 Map Length

The `.len()` method returns the number of key-value pairs:

```vibe
ages := {"Alice": 30, "Bob": 25}
count := ages.len()
```

`count` is `2`.

### 9.3.8 Deterministic Iteration Order

One of VibeLang's strongest guarantees about maps is **deterministic iteration
order**. Maps iterate in insertion order — the order in which keys were first
added:

```vibe
mut m := {"first": 1, "second": 2, "third": 3}
for (key, value) in m {
    print(key + ": " + value.to_str())
}
```

This always prints:

```
first: 1
second: 2
third: 3
```

This guarantee holds even after updates and removals. Updating an existing key
does *not* change its position in the iteration order. Removing a key and
re-inserting it places it at the end.

Why does this matter? Deterministic iteration is essential for reproducible
builds, deterministic test output, and predictable serialization. Many languages
(notably older versions of Python, or Go) have non-deterministic map iteration,
which leads to flaky tests and hard-to-reproduce bugs. VibeLang eliminates this
class of problems by design.

### 9.3.9 Maps and the `alloc` Effect

Like lists, maps allocate heap memory. Functions that create or modify maps
should declare the `alloc` effect:

```vibe
pub word_count(words: List<Str>) -> Map<Str, Int> {
    @effect alloc

    mut counts : Map<Str, Int> := {}
    for word in words {
        if counts.contains(word) {
            counts.set(word, counts.get(word) + 1)
        } else {
            counts.set(word, 1)
        }
    }
    counts
}
```

### 9.3.10 A Complete Map Example

Here is a program that reads a list of scores, groups them by grade bracket, and
reports the counts:

```vibe
pub grade_bracket(score: Int) -> Str {
    if score >= 90 {
        "A"
    } else if score >= 80 {
        "B"
    } else if score >= 70 {
        "C"
    } else if score >= 60 {
        "D"
    } else {
        "F"
    }
}

pub grade_distribution(scores: List<Int>) -> Map<Str, Int> {
    @effect alloc
    @ensure .len() <= 5

    mut dist : Map<Str, Int> := {}
    for score in scores {
        bracket := grade_bracket(score)
        if dist.contains(bracket) {
            dist.set(bracket, dist.get(bracket) + 1)
        } else {
            dist.set(bracket, 1)
        }
    }
    dist
}

pub main() -> Int {
    @effect alloc

    scores := [95, 87, 72, 63, 91, 88, 55, 78, 82, 96]
    dist := grade_distribution(scores)

    for (grade, count) in dist {
        print(grade + ": " + count.to_str())
    }

    0
}
```

Because maps iterate in insertion order, the output order depends on which grade
bracket appears first in the input list. For the scores above, `"A"` appears
first (95), so the output begins with `A: 3`.

## 9.4 Choosing the Right Collection

### 9.4.1 When to Use `List<T>`

Use a list when:

- **Order matters and you access by position.** Lists maintain insertion order
  and support O(1) indexed access.
- **You need to process elements sequentially.** `for ... in` iteration over
  lists is the most natural loop pattern.
- **You need sorting.** Lists support in-place sorting; maps do not.
- **Duplicates are allowed.** Lists happily store the same value multiple times.

### 9.4.2 When to Use `Map<K,V>`

Use a map when:

- **You need fast lookup by key.** `.get(key)` and `.contains(key)` are O(1)
  average-case operations.
- **You're associating data.** Names to ages, words to counts, IDs to records —
  these are natural map use cases.
- **You need uniqueness by key.** Maps enforce unique keys automatically;
  setting the same key overwrites the previous value.

### 9.4.3 Performance Characteristics

| Operation          | `List<T>`      | `Map<K,V>`     |
|--------------------|----------------|----------------|
| Index access       | O(1)           | N/A            |
| Key lookup         | O(n) scan      | O(1) average   |
| Append             | O(1) amortized | N/A            |
| Insert key-value   | N/A            | O(1) amortized |
| Remove by index    | O(n)           | N/A            |
| Remove by key      | N/A            | O(1) average   |
| Iteration          | O(n)           | O(n)           |
| Sort               | O(n log n)     | Not supported  |
| Memory overhead    | Low            | Moderate       |

### 9.4.4 Memory Considerations

Lists store elements contiguously in memory, which is cache-friendly for
sequential access. Maps use a hash table internally, which has higher per-entry
overhead but provides fast key-based access.

For small collections (fewer than ~20 elements), the performance difference is
negligible. Choose based on semantics — does your data have keys, or is it a
sequence?

For large collections, be aware that both lists and maps trigger the `alloc`
effect and participate in garbage collection. If you're building a
performance-critical pipeline, the effect system helps you identify which
functions allocate and where GC pressure originates.

### 9.4.5 Combining Collections

Lists and maps compose naturally. A common pattern is to use a list of keys to
control processing order over a map:

```vibe
pub process_in_priority_order(
    priorities: List<Str>,
    tasks: Map<Str, Int>
) -> Int {
    @require priorities.len() > 0
    mut total := 0
    for key in priorities {
        if tasks.contains(key) {
            total = total + tasks.get(key)
        }
    }
    total
}
```

Another common pattern is building a map from a list:

```vibe
pub index_by_position(items: List<Str>) -> Map<Int, Str> {
    @effect alloc

    mut index : Map<Int, Str> := {}
    mut i := 0
    for item in items {
        index.set(i, item)
        i = i + 1
    }
    index
}
```

## 9.5 Summary

VibeLang's three built-in collection types cover the vast majority of data
organization needs:

- **`Str`** is an immutable, UTF-8 encoded byte sequence. It supports
  concatenation with `+`, byte-length via `.len()`, slicing with boundary
  safety, and a rich set of text operations. Strings are immutable by design,
  which makes them safe to share across task boundaries.

- **`List<T>`** is an ordered, growable sequence. It supports indexed access,
  appending, sorting, and taking subsets. Mutation requires a `mut` binding.
  Lists are the natural choice for ordered data and sequential processing.

- **`Map<K,V>`** is a key-value associative collection with deterministic
  insertion-order iteration. It supports fast key lookup, insertion, removal,
  and containment checks. Deterministic iteration eliminates flaky tests and
  non-reproducible behavior.

All three types allocate heap memory and participate in the `alloc` effect
system. All three are immutable by default — you need `mut` bindings to modify
them. And all three work naturally with VibeLang's contract system, enabling
preconditions and postconditions on the data they contain.

In the next chapter, we'll explore how to organize code that uses these
collections into modules and packages — the building blocks of larger VibeLang
programs.
