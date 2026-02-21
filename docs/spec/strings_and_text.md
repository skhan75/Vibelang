# VibeLang Strings and Text Semantics (v1.0 Target)

Status: normative target.

## Canonical String Representation

- `Str` stores UTF-8 encoded text.
- String length APIs must clearly distinguish:
  - bytes
  - Unicode scalar values (code points)
  - grapheme clusters (if supported by stdlib helper).

Default `len(str)` returns byte length unless documented otherwise.

## Literal and Escape Rules

String literal syntax:

```txt
"hello\nworld"
```

Supported escapes (baseline):

- `\\` backslash
- `\"` quote
- `\n`, `\r`, `\t`
- `\u{...}` Unicode scalar escape

Invalid escape sequences are compile-time parse errors.

## Character Literals

- Character literals represent one Unicode scalar value.
- Multi-scalar graphemes are not valid char literals.

## Indexing and Slicing

- Direct string index by byte offset is allowed only where offset is valid
  UTF-8 boundary.
- Invalid boundary access is deterministic runtime error (or compile-time error
  if provable).
- Slice semantics MUST specify boundaries as byte offsets unless API explicitly
  declares code-point/grapheme behavior.

## Normalization

- Language runtime does not implicitly normalize Unicode text.
- Equality is byte-level equality unless API explicitly performs normalization.

## Concatenation and Builders

- `+` may concatenate strings.
- Repeated concatenation in loops SHOULD use builder APIs for predictable
  allocation behavior.
- Builder APIs (if present) MUST define deterministic growth strategy semantics.

## Interop and Encoding Boundaries

- FFI/text boundary APIs MUST specify encoding assumptions.
- Invalid UTF-8 at foreign boundary must be handled deterministically:
  - rejection with error, or
  - explicit lossy conversion API.

## Ordering and Equality

- `==` and `!=` on `Str` operate on byte-equality.
- Lexicographic comparison, if supported, is byte-order based unless explicit
  locale-aware API is invoked.

## Performance Guarantees

- String operations must document allocation behavior where material:
  - copy-on-write vs eager copy
  - amortized complexity for append/builder operations.

## Deferred Notes

- Locale-sensitive collation and grapheme-aware default indexing are deferred.
