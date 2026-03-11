// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

pub fn format_source(input: &str) -> String {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = Vec::new();
    let mut blank_run = 0usize;

    for raw in normalized.split('\n') {
        let line = raw.replace('\t', "  ");
        let trimmed = line.trim_end().to_string();
        if trimmed.is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        lines.push(trimmed);
    }

    while lines.first().is_some_and(String::is_empty) {
        lines.remove(0);
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }

    if lines.is_empty() {
        return String::new();
    }

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

pub fn needs_formatting(input: &str) -> bool {
    format_source(input) != input
}

#[cfg(test)]
mod tests {
    use super::{format_source, needs_formatting};

    #[test]
    fn formatter_is_idempotent() {
        let src = "pub main() -> Int {\n\tprintln(\"hi\")    \n\n\n  0\n}\n";
        let once = format_source(src);
        let twice = format_source(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn formatter_normalizes_blank_lines_and_trailing_whitespace() {
        let src = "\n\nfoo() {\n  1    \n\n\n\n  2\t\n}\n\n";
        let out = format_source(src);
        assert_eq!(out, "foo() {\n  1\n\n  2\n}\n");
    }

    #[test]
    fn needs_formatting_matches_formatter_result() {
        let src = "a() {\n  1\n}\n";
        assert!(!needs_formatting(src));
        assert!(needs_formatting("a() {\n\t1\n}\n"));
    }
}
