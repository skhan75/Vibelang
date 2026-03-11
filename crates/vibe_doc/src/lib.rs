// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocItem {
    pub name: String,
    pub signature: String,
    pub source_line: usize,
    pub intent: Option<String>,
    pub effects: Vec<String>,
    pub requires: Vec<String>,
    pub ensures: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct PendingAnnots {
    intent: Option<String>,
    effects: Vec<String>,
    requires: Vec<String>,
    ensures: Vec<String>,
    examples: Vec<String>,
}

pub fn extract_docs(source: &str) -> Vec<DocItem> {
    let fn_re = Regex::new(r"^(?:pub\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\(").expect("valid regex");
    let mut out = Vec::new();
    let mut pending = PendingAnnots::default();
    let mut in_examples = false;

    for (idx, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        if in_examples {
            if line == "}" {
                in_examples = false;
            } else if !line.starts_with("@examples") {
                pending.examples.push(line.to_string());
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("@intent ") {
            pending.intent = Some(clean_quoted(rest));
            continue;
        }
        if let Some(rest) = line.strip_prefix("@effect ") {
            pending.effects.push(rest.trim().to_string());
            continue;
        }
        if let Some(rest) = line.strip_prefix("@require ") {
            pending.requires.push(rest.trim().to_string());
            continue;
        }
        if let Some(rest) = line.strip_prefix("@ensure ") {
            pending.ensures.push(rest.trim().to_string());
            continue;
        }
        if line.starts_with("@examples") {
            in_examples = true;
            continue;
        }

        let Some(caps) = fn_re.captures(line) else {
            continue;
        };
        let name = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        out.push(DocItem {
            name,
            signature: line.trim_end_matches('{').trim().to_string(),
            source_line: idx + 1,
            intent: pending.intent.take(),
            effects: std::mem::take(&mut pending.effects),
            requires: std::mem::take(&mut pending.requires),
            ensures: std::mem::take(&mut pending.ensures),
            examples: std::mem::take(&mut pending.examples),
        });
    }

    out
}

pub fn render_markdown(module_name: &str, items: &[DocItem]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# API: {module_name}\n\n"));
    if items.is_empty() {
        out.push_str("_No documentable functions found._\n");
        return out;
    }

    for item in items {
        out.push_str(&format!("## `{}`\n\n", item.signature));
        out.push_str(&format!("- Source: line {}\n", item.source_line));
        if let Some(intent) = &item.intent {
            out.push_str(&format!("- Intent: {intent}\n"));
        }
        if !item.effects.is_empty() {
            out.push_str(&format!("- Effects: {}\n", item.effects.join(", ")));
        }
        if !item.requires.is_empty() {
            out.push_str(&format!("- Requires: {}\n", item.requires.join(" | ")));
        }
        if !item.ensures.is_empty() {
            out.push_str(&format!("- Ensures: {}\n", item.ensures.join(" | ")));
        }
        if !item.examples.is_empty() {
            out.push_str("- Examples:\n");
            for example in &item.examples {
                out.push_str(&format!("  - `{example}`\n"));
            }
        }
        out.push('\n');
    }
    out
}

fn clean_quoted(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::{extract_docs, render_markdown};

    #[test]
    fn extracts_annotations_and_functions() {
        let src = r#"
@intent "sum numbers"
@effect alloc
sum(xs: List<Int>) -> Int {
  @examples {
    sum([1,2]) => 3
  }
  0
}
"#;
        let docs = extract_docs(src);
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "sum");
        assert_eq!(docs[0].intent.as_deref(), Some("sum numbers"));
        assert_eq!(docs[0].effects, vec!["alloc"]);
    }

    #[test]
    fn markdown_render_is_stable() {
        let src = r#"
@intent "hello"
pub main() -> Int {
  0
}
"#;
        let docs = extract_docs(src);
        let rendered = render_markdown("sample", &docs);
        assert!(rendered.contains("# API: sample"));
        assert!(rendered.contains("## `pub main() -> Int`"));
        assert!(rendered.contains("- Intent: hello"));
    }
}
