// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Enforcement for the network-byte contract behind VL-HTTP-01 / VL-HTTP-02.
//!
//! THE RULE: trust is a property of the CODE PATH, not of the value. A VibeLang
//! `Str` is a bare NUL-terminated `char*` with no header, so there is nowhere to
//! record "these bytes came from a stranger" -- unlike heap containers, which
//! carry a tag word. The contract therefore lives at the call site:
//!
//!   If the `Str` could have come from `net.read` / `ws.read_frame` / an HTTP
//!   response / a decoder fed by those, use `text.slice_bytes` / `text.byte_at`.
//!   If it could only have come from a literal or from your own arithmetic, use
//!   the `s[a:b]` sugar and let it panic.
//!
//! `MODULES` below is the audit surface: the set of stdlib modules that handle
//! peer bytes. This test fails the build if any of them reaches for the panicking
//! sugar, so the choice is not a doc paragraph a contributor may skip.

use std::fs;
use std::path::PathBuf;

/// stdlib modules whose inputs are bytes received from a socket.
///
/// `std.http_router` qualifies because every `Str` it slices is `req.headers` or
/// `req.query`, i.e. slots 3 and 2 of `vibe_http_server_parse_request`, which
/// carves them straight out of the raw request with no validation.
///
/// Deliberately NOT listed: `std.path`, which also uses slicing and byte
/// indexing but on filesystem paths rather than socket bytes. A path assembled
/// from a URL segment would move it into scope and deserves its own review
/// rather than a silent assumption.
const MODULES: &[&str] = &["stdlib/std/http_router.yb"];

#[test]
fn network_facing_modules_do_not_use_panicking_slice_sugar() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for module in MODULES {
        let path = root.join(module);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read network-facing module {}: {e}", path.display()));

        let scrubbed = strip_strings_and_comments(&source);
        for (idx, (code, raw)) in scrubbed.lines().zip(source.lines()).enumerate() {
            if let Some(kind) = bracket_sugar_kind(code) {
                violations.push(format!(
                    "{module}:{}: {kind} on a network-facing path -- use text.slice_bytes / \
                     text.byte_at instead (the sugar calls vibe_panic on a non-UTF-8 boundary, \
                     which aborts the whole server): {}",
                    idx + 1,
                    raw.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "network-facing stdlib modules must not use panicking slice/index sugar:\n{}",
        violations.join("\n")
    );
}

/// Guard on the guard: the detector must actually fire, otherwise an empty
/// `violations` list would prove nothing.
#[test]
fn bracket_sugar_detector_recognises_the_patterns_it_bans() {
    for banned in [
        "  rest := line[colon + 1:line.len()]",
        "  rem := headers[pos:total]",
        "  k := part[0:eq]",
        "  b := s[i]",
        "  key := text.trim(line[0:colon])",
    ] {
        assert!(
            bracket_sugar_kind(&strip_strings_and_comments(banned)).is_some(),
            "detector missed banned pattern: {banned}"
        );
    }

    // The scrubber must survive the shapes that actually appear in stdlib source.
    assert!(
        bracket_sugar_kind(&strip_strings_and_comments(
            "/**\n * do not use s[a:b] here\n */\ncode()"
        ))
        .is_none(),
        "block comment prose must not count as a violation"
    );
    assert!(
        bracket_sugar_kind(&strip_strings_and_comments(
            "/* comment */\n  rem := headers[pos:total]"
        ))
        .is_some(),
        "code after a block comment must still be scanned"
    );

    for allowed in [
        "  rest := text.slice_bytes(line, colon + 1, line.len())",
        "  b := text.byte_at(s, i)",
        "  parts: List<Str> = text.split(s)",
        "  println(\"a[0:1] inside a string literal\")",
        "  // headers[pos:total] in a comment",
        "  m: Map<Str, Int> = {}",
    ] {
        assert!(
            bracket_sugar_kind(&strip_strings_and_comments(allowed)).is_none(),
            "detector false-positived on allowed line: {allowed}"
        );
    }
}

/// Blank out string literals, line comments and block comments so the scan
/// cannot trip on bracket characters that are only data or prose. Newlines are
/// preserved so line numbers still line up with the original source.
fn strip_strings_and_comments(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0usize;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut block_depth = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        if ch == '\n' {
            in_line_comment = false;
            in_string = false; // .yb string literals do not span lines
            out.push('\n');
            i += 1;
            continue;
        }
        if in_line_comment {
            out.push(' ');
            i += 1;
            continue;
        }
        if block_depth > 0 {
            if ch == '*' && next == Some('/') {
                block_depth -= 1;
                out.push_str("  ");
                i += 2;
                continue;
            }
            if ch == '/' && next == Some('*') {
                block_depth += 1;
                out.push_str("  ");
                i += 2;
                continue;
            }
            out.push(' ');
            i += 1;
            continue;
        }
        if in_string {
            if ch == '\\' {
                out.push(' ');
                if next.is_some() {
                    out.push(' ');
                    i += 1;
                }
            } else {
                if ch == '"' {
                    in_string = false;
                }
                out.push(' ');
            }
            i += 1;
            continue;
        }
        if ch == '/' && next == Some('/') {
            in_line_comment = true;
            out.push_str("  ");
            i += 2;
            continue;
        }
        if ch == '/' && next == Some('*') {
            block_depth = 1;
            out.push_str("  ");
            i += 2;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(' ');
            i += 1;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

/// Classify a code line as slice sugar (`x[a:b]`), index sugar (`x[i]`), or
/// neither. Only bracket groups directly attached to an identifier or a closing
/// paren count, so generic parameters and map/list literals are ignored.
fn bracket_sugar_kind(code: &str) -> Option<&'static str> {
    let chars: Vec<char> = code.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch != '[' {
            continue;
        }
        let attached = chars[..i]
            .iter()
            .rev()
            .find(|c| !c.is_whitespace())
            .is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == ')');
        if !attached {
            continue;
        }
        let mut depth = 0usize;
        let mut has_colon = false;
        for &c in &chars[i..] {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(if has_colon {
                            "slice sugar `[a:b]`"
                        } else {
                            "byte-index sugar `[i]`"
                        });
                    }
                }
                ':' if depth == 1 => has_colon = true,
                _ => {}
            }
        }
    }
    None
}

/// VL-HTTP-02's amplifier: in `scan_query_for_key` the value must be sliced only
/// after the key matched, so a parameter the application never reads cannot
/// influence the process at all. With `slice_bytes` in place this ordering is no
/// longer observable through the public API, so it is pinned structurally --
/// defence in depth, checked where it can actually be checked.
#[test]
fn query_value_is_sliced_only_after_the_key_matches() {
    let source = fs::read_to_string(workspace_root().join("stdlib/std/http_router.yb"))
        .expect("read http_router.yb");
    let body = source
        .split("scan_query_for_key(query: Str")
        .nth(1)
        .expect("scan_query_for_key definition");

    let key_test = body
        .find("if k == want")
        .expect("key comparison `if k == want`");
    let value_slice = body
        .find("text.slice_bytes(part, eq + 1")
        .expect("value slice `text.slice_bytes(part, eq + 1, ...)`");

    assert!(
        value_slice > key_test,
        "the query value slice must appear after the `k == want` test, otherwise every \
         parameter -- including ones the application never reads -- gets sliced"
    );
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve workspace root")
}
