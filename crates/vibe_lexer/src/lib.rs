use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Module,
    Import,
    Pub,
    Type,
    If,
    Else,
    For,
    While,
    Repeat,
    Match,
    Return,
    Go,
    Select,
    Case,
    After,
    Closed,
    Default,
    In,
    True,
    False,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    IntLit,
    FloatLit,
    StringLit,
    At,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Question,
    Arrow,
    FatArrow,
    Assign,
    Bind,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Keyword(Keyword),
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

pub fn lex(source: &str) -> (Vec<Token>, Diagnostics) {
    let mut lx = Lexer::new(source);
    lx.run();
    (lx.tokens, lx.diags)
}

struct Lexer {
    chars: Vec<char>,
    idx: usize,
    line: usize,
    col: usize,
    tokens: Vec<Token>,
    diags: Diagnostics,
}

impl Lexer {
    fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            idx: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            diags: Diagnostics::default(),
        }
    }

    fn run(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.bump();
                }
                '\n' => {
                    let span = Span::new(self.line, self.col, self.line, self.col);
                    self.bump_newline();
                    self.tokens.push(Token {
                        kind: TokenKind::Newline,
                        lexeme: "\n".to_string(),
                        span,
                    });
                }
                '/' if self.peek_next() == Some('/') => {
                    self.consume_comment();
                }
                '"' => self.lex_string(),
                '0'..='9' => self.lex_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.lex_ident_or_keyword(),
                _ => self.lex_symbol(),
            }
        }
        let eof_span = Span::new(self.line, self.col, self.line, self.col);
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            span: eof_span,
        });
    }

    fn lex_string(&mut self) {
        let (line, col) = (self.line, self.col);
        self.bump(); // opening quote
        let mut value = String::new();
        let mut terminated = false;
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.bump();
                    terminated = true;
                    break;
                }
                '\n' => {
                    self.bump_newline();
                    value.push('\n');
                }
                '\\' => {
                    self.bump();
                    if let Some(escaped) = self.peek() {
                        let resolved = match escaped {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '"' => '"',
                            '\\' => '\\',
                            other => other,
                        };
                        self.bump();
                        value.push(resolved);
                    }
                }
                c => {
                    self.bump();
                    value.push(c);
                }
            }
        }
        let end_col = self.col.saturating_sub(1);
        let span = Span::new(line, col, self.line, end_col);
        if !terminated {
            self.diags.push(Diagnostic::new(
                "E1002",
                Severity::Error,
                "unterminated string literal",
                span,
            ));
        }
        self.tokens.push(Token {
            kind: TokenKind::StringLit,
            lexeme: value,
            span,
        });
    }

    fn lex_number(&mut self) {
        let (line, col) = (self.line, self.col);
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.bump();
            } else {
                break;
            }
        }

        let mut kind = TokenKind::IntLit;
        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            kind = TokenKind::FloatLit;
            text.push('.');
            self.bump();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    text.push(ch);
                    self.bump();
                } else {
                    break;
                }
            }
        }
        let span = Span::new(line, col, self.line, self.col.saturating_sub(1));
        self.tokens.push(Token {
            kind,
            lexeme: text,
            span,
        });
    }

    fn lex_ident_or_keyword(&mut self) {
        let (line, col) = (self.line, self.col);
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(ch);
                self.bump();
            } else {
                break;
            }
        }
        let kind = match keyword_of(&text) {
            Some(k) => TokenKind::Keyword(k),
            None => TokenKind::Ident,
        };
        let span = Span::new(line, col, self.line, self.col.saturating_sub(1));
        self.tokens.push(Token {
            kind,
            lexeme: text,
            span,
        });
    }

    fn lex_symbol(&mut self) {
        let (line, col) = (self.line, self.col);
        let ch = self.bump().unwrap_or_default();
        let (kind, lexeme) = match ch {
            '@' => (TokenKind::At, "@".to_string()),
            '{' => (TokenKind::LBrace, "{".to_string()),
            '}' => (TokenKind::RBrace, "}".to_string()),
            '(' => (TokenKind::LParen, "(".to_string()),
            ')' => (TokenKind::RParen, ")".to_string()),
            '[' => (TokenKind::LBracket, "[".to_string()),
            ']' => (TokenKind::RBracket, "]".to_string()),
            ',' => (TokenKind::Comma, ",".to_string()),
            '.' => (TokenKind::Dot, ".".to_string()),
            '?' => (TokenKind::Question, "?".to_string()),
            '+' => (TokenKind::Plus, "+".to_string()),
            '*' => (TokenKind::Star, "*".to_string()),
            '/' => (TokenKind::Slash, "/".to_string()),
            ':' if self.peek() == Some('=') => {
                self.bump();
                (TokenKind::Bind, ":=".to_string())
            }
            ':' => (TokenKind::Colon, ":".to_string()),
            '-' if self.peek() == Some('>') => {
                self.bump();
                (TokenKind::Arrow, "->".to_string())
            }
            '-' => (TokenKind::Minus, "-".to_string()),
            '=' if self.peek() == Some('>') => {
                self.bump();
                (TokenKind::FatArrow, "=>".to_string())
            }
            '=' if self.peek() == Some('=') => {
                self.bump();
                (TokenKind::EqEq, "==".to_string())
            }
            '=' => (TokenKind::Assign, "=".to_string()),
            '!' if self.peek() == Some('=') => {
                self.bump();
                (TokenKind::NotEq, "!=".to_string())
            }
            '!' => (TokenKind::Bang, "!".to_string()),
            '<' if self.peek() == Some('=') => {
                self.bump();
                (TokenKind::Le, "<=".to_string())
            }
            '<' => (TokenKind::Lt, "<".to_string()),
            '>' if self.peek() == Some('=') => {
                self.bump();
                (TokenKind::Ge, ">=".to_string())
            }
            '>' => (TokenKind::Gt, ">".to_string()),
            other => {
                let span = Span::new(line, col, self.line, self.col.saturating_sub(1));
                self.diags.push(Diagnostic::new(
                    "E1001",
                    Severity::Error,
                    format!("unexpected character `{other}`"),
                    span,
                ));
                return;
            }
        };
        let span = Span::new(line, col, self.line, self.col.saturating_sub(1));
        self.tokens.push(Token { kind, lexeme, span });
    }

    fn consume_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.idx).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.idx + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.idx += 1;
        self.col += 1;
        Some(ch)
    }

    fn bump_newline(&mut self) {
        self.idx += 1;
        self.line += 1;
        self.col = 1;
    }
}

fn keyword_of(text: &str) -> Option<Keyword> {
    Some(match text {
        "module" => Keyword::Module,
        "import" => Keyword::Import,
        "pub" => Keyword::Pub,
        "type" => Keyword::Type,
        "if" => Keyword::If,
        "else" => Keyword::Else,
        "for" => Keyword::For,
        "while" => Keyword::While,
        "repeat" => Keyword::Repeat,
        "match" => Keyword::Match,
        "return" => Keyword::Return,
        "go" => Keyword::Go,
        "select" => Keyword::Select,
        "case" => Keyword::Case,
        "after" => Keyword::After,
        "closed" => Keyword::Closed,
        "default" => Keyword::Default,
        "in" => Keyword::In,
        "true" => Keyword::True,
        "false" => Keyword::False,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::lex;

    #[test]
    fn lexes_basic_tokens() {
        let src = r#"module app
fn1(a, b) { x := 1 }
"#;
        let (tokens, diags) = lex(src);
        assert!(!tokens.is_empty());
        assert!(!diags.has_errors());
    }
}
