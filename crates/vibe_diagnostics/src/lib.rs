use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Span {
    pub line_start: usize,
    pub col_start: usize,
    pub line_end: usize,
    pub col_end: usize,
}

impl Span {
    pub fn new(line_start: usize, col_start: usize, line_end: usize, col_end: usize) -> Self {
        Self {
            line_start,
            col_start,
            line_end,
            col_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedSpan {
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub related: Vec<RelatedSpan>,
}

impl Diagnostic {
    pub fn new(
        code: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }

    pub fn with_related(mut self, message: impl Into<String>, span: Span) -> Self {
        self.related.push(RelatedSpan {
            message: message.into(),
            span,
        });
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn push(&mut self, d: Diagnostic) {
        self.items.push(d);
    }

    pub fn extend(&mut self, mut ds: Vec<Diagnostic>) {
        self.items.append(&mut ds);
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.items
    }

    pub fn into_sorted(mut self) -> Vec<Diagnostic> {
        self.items.sort_by(|a, b| {
            (
                a.span.line_start,
                a.span.col_start,
                a.span.line_end,
                a.span.col_end,
                a.code.as_str(),
                severity_rank(a.severity),
            )
                .cmp(&(
                    b.span.line_start,
                    b.span.col_start,
                    b.span.line_end,
                    b.span.col_end,
                    b.code.as_str(),
                    severity_rank(b.severity),
                ))
        });
        self.items
    }

    pub fn sorted(&self) -> Vec<Diagnostic> {
        self.clone().into_sorted()
    }

    pub fn to_golden(&self) -> String {
        let mut out = String::new();
        for d in self.sorted() {
            let _ = fmt::write(
                &mut out,
                format_args!(
                    "{}: {}: {} @ {}:{}-{}:{}\n",
                    d.code,
                    d.severity,
                    d.message,
                    d.span.line_start,
                    d.span.col_start,
                    d.span.line_end,
                    d.span.col_end
                ),
            );
            for rel in d.related {
                let _ = fmt::write(
                    &mut out,
                    format_args!(
                        "  related: {} @ {}:{}-{}:{}\n",
                        rel.message,
                        rel.span.line_start,
                        rel.span.col_start,
                        rel.span.line_end,
                        rel.span.col_end
                    ),
                );
            }
        }
        out
    }
}

fn severity_rank(s: Severity) -> usize {
    match s {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}
