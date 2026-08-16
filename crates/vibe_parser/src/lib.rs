// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use vibe_ast::{
    BinaryOp, Contract, Declaration, EnumDecl, EnumVariantDecl, ExampleCase, Expr, FileAst,
    FunctionDecl, MatchArm, Param, SelectCase, SelectPattern, Stmt, TypeDecl, TypeField, TypeRef,
    UnaryOp,
};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};
use vibe_lexer::{lex, Keyword, Token, TokenKind};

#[derive(Debug)]
pub struct ParseOutput {
    pub ast: FileAst,
    pub diagnostics: Diagnostics,
}

pub fn parse_source(source: &str) -> ParseOutput {
    let (tokens, mut diagnostics) = lex(source);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_file();
    diagnostics.extend(parser.diagnostics.into_sorted());
    ParseOutput { ast, diagnostics }
}

/// Maximum simultaneous expression-parse recursion frames. Every nested
/// `(`/`[`/`{` level and every chained unary operator adds one frame, so
/// source can nest expressions at most `MAX_EXPR_NESTING_DEPTH - 1` levels
/// deep. Sizing (measured 2026-08): one frame costs ~3.3 KiB in release
/// and ~15.6 KiB in debug, so 256 frames peak near 0.85 MiB / 3.9 MiB —
/// comfortably inside the 8 MiB main thread that `vibe check`, `vibe
/// build`, and the (single-threaded) LSP all parse on, and inside 2 MiB
/// worker threads in release. Unguarded parsing overflowed the stack near
/// 3000 levels (release) / 135 levels (debug, 2 MiB thread). The deepest
/// nesting observed anywhere in the shipped corpus (examples, stdlib,
/// benchmarks, book, tests: 270 files) is 17, so 256 leaves >10x headroom
/// over real code. Tests that parse near the limit must run on a thread
/// with a large explicit stack (2 MiB debug test threads fit only ~135
/// frames); see `on_big_stack` in this file's tests.
const MAX_EXPR_NESTING_DEPTH: usize = 256;

struct Parser {
    tokens: Vec<Token>,
    idx: usize,
    diagnostics: Diagnostics,
    /// Current expression-parse recursion depth (see `MAX_EXPR_NESTING_DEPTH`).
    expr_depth: usize,
    /// Set while unwinding from an expression-nesting overflow (E1415);
    /// suppresses follow-on "expected X" noise until the expression that
    /// overflowed has been fully abandoned. Cleared when `expr_depth`
    /// returns to zero.
    expr_depth_exceeded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopToken {
    Newline,
    Comma,
    Colon,
    RParen,
    RBrace,
    RBracket,
    FatArrow,
    LBrace,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            idx: 0,
            diagnostics: Diagnostics::default(),
            expr_depth: 0,
            expr_depth_exceeded: false,
        }
    }

    fn parse_file(&mut self) -> FileAst {
        let mut file = FileAst::default();
        self.consume_newlines();

        if self.at_keyword(Keyword::Module) {
            self.bump();
            file.module = Some(self.parse_qualified_ident());
            self.consume_line_end();
        }

        while self.at_keyword(Keyword::Import) {
            self.bump();
            file.imports.push(self.parse_qualified_ident());
            self.consume_line_end();
        }

        while !self.is_eof() {
            self.consume_newlines();
            if self.is_eof() {
                break;
            }
            if let Some(decl) = self.parse_declaration() {
                file.declarations.push(decl);
            } else {
                self.sync_to_decl();
            }
        }
        file
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        let mut is_public = false;
        if self.at_keyword(Keyword::Pub) {
            is_public = true;
            self.bump();
        }
        if self.at_keyword(Keyword::Type) {
            let mut decl = self.parse_type_decl();
            decl.is_public = is_public;
            Some(Declaration::Type(decl))
        } else if self.at_keyword(Keyword::Enum) {
            let mut decl = self.parse_enum_decl();
            decl.is_public = is_public;
            Some(Declaration::Enum(decl))
        } else if self.at_ident() {
            let mut func = self.parse_function();
            func.is_public = is_public;
            Some(Declaration::Function(func))
        } else {
            let span = self.peek().span;
            self.diagnostics.push(Diagnostic::new(
                "E1101",
                Severity::Error,
                "expected declaration",
                span,
            ));
            None
        }
    }

    fn parse_type_decl(&mut self) -> TypeDecl {
        let start = self.bump().span;
        let name = self.expect_ident("E1102", "expected type name after `type`");
        self.expect(
            TokenKind::LBrace,
            "E1105",
            "expected `{` to start type body",
        );
        let mut fields = Vec::new();
        self.consume_newlines();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            let field_name = self.expect_ident("E1140", "expected field name");
            self.expect(TokenKind::Colon, "E1141", "expected `:` after field name");
            let ty = self.parse_type_ref_until(&[TokenKind::Comma, TokenKind::RBrace]);
            fields.push(TypeField {
                name: field_name,
                ty,
            });
            if self.match_kind(&TokenKind::Comma) {
                self.consume_newlines();
            } else {
                break;
            }
        }
        self.consume_newlines();
        let end = self.expect(
            TokenKind::RBrace,
            "E1106",
            "expected `}` to close type body",
        );
        TypeDecl {
            is_public: false,
            name,
            fields,
            span: Span::new(start.line_start, start.col_start, end.line_end, end.col_end),
        }
    }

    fn parse_enum_decl(&mut self) -> EnumDecl {
        let start = self.bump().span;
        let name = self.expect_ident("E1102", "expected enum name after `enum`");
        self.expect(
            TokenKind::LBrace,
            "E1105",
            "expected `{` to start enum body",
        );
        let mut variants = Vec::new();
        self.consume_newlines();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            let v_start = self.peek().span;
            let variant = self.expect_ident("E1142", "expected variant name");
            let (fields, variant_span) = if self.at(&TokenKind::LBrace) {
                self.bump();
                let mut fs = Vec::new();
                self.consume_newlines();
                while !self.at(&TokenKind::RBrace) && !self.is_eof() {
                    let field_name =
                        self.expect_ident("E1140", "expected field name in enum variant");
                    self.expect(
                        TokenKind::Colon,
                        "E1141",
                        "expected `:` after field name in enum variant",
                    );
                    let ty = self.parse_type_ref_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    fs.push(TypeField {
                        name: field_name,
                        ty,
                    });
                    if self.match_kind(&TokenKind::Comma) {
                        self.consume_newlines();
                    } else {
                        break;
                    }
                }
                self.consume_newlines();
                let fe = self.expect(
                    TokenKind::RBrace,
                    "E1106",
                    "expected `}` to close enum variant payload",
                );
                (
                    fs,
                    Span::new(
                        v_start.line_start,
                        v_start.col_start,
                        fe.line_end,
                        fe.col_end,
                    ),
                )
            } else {
                (
                    Vec::new(),
                    Span::new(
                        v_start.line_start,
                        v_start.col_start,
                        v_start.line_end,
                        v_start.col_end,
                    ),
                )
            };
            variants.push(EnumVariantDecl {
                name: variant,
                fields,
                span: variant_span,
            });
            if self.match_kind(&TokenKind::Comma) {
                self.consume_newlines();
            } else {
                break;
            }
        }
        self.consume_newlines();
        let end = self.expect(
            TokenKind::RBrace,
            "E1106",
            "expected `}` to close enum body",
        );
        EnumDecl {
            is_public: false,
            name,
            variants,
            span: Span::new(start.line_start, start.col_start, end.line_end, end.col_end),
        }
    }

    fn parse_optional_type_params(&mut self) -> Vec<String> {
        if !self.match_kind(&TokenKind::Lt) {
            return Vec::new();
        }
        let mut params = Vec::new();
        loop {
            if self.at(&TokenKind::Gt) {
                break;
            }
            params.push(self.expect_ident("E1102a", "expected type parameter name after `<`"));
            if self.match_kind(&TokenKind::Comma) {
                continue;
            }
            break;
        }
        self.expect(
            TokenKind::Gt,
            "E1102b",
            "expected `>` to close type parameter list",
        );
        params
    }

    fn parse_function(&mut self) -> FunctionDecl {
        let start = self.peek().span;
        let name = self.expect_ident("E1102", "expected function name");
        let type_params = self.parse_optional_type_params();
        self.expect(
            TokenKind::LParen,
            "E1103",
            "expected `(` after function name",
        );
        let params = self.parse_params();
        self.expect(
            TokenKind::RParen,
            "E1104",
            "expected `)` after parameter list",
        );

        let return_type = if self.match_kind(&TokenKind::Arrow) {
            Some(self.parse_type_ref_until(&[TokenKind::LBrace]))
        } else {
            None
        };

        self.expect(
            TokenKind::LBrace,
            "E1105",
            "expected `{` to start function body",
        );

        let mut contracts = Vec::new();
        let mut body = Vec::new();
        let mut seen_exec = false;
        self.consume_newlines();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            let before = self.idx;
            self.consume_newlines();
            if self.at(&TokenKind::RBrace) || self.is_eof() {
                break;
            }
            if !seen_exec && self.at(&TokenKind::At) {
                contracts.push(self.parse_contract());
                self.consume_newlines();
                continue;
            }
            if seen_exec && self.at(&TokenKind::At) {
                self.diagnostics.push(Diagnostic::new(
                    "E1309",
                    Severity::Error,
                    "invalid contract position: annotations must appear before executable statements",
                    self.peek().span,
                ));
                let _ = self.parse_contract();
                self.consume_newlines();
                continue;
            }
            seen_exec = true;
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            } else {
                self.sync_to_stmt_boundary();
            }
            self.consume_newlines();
            if self.idx == before && !self.at(&TokenKind::RBrace) && !self.is_eof() {
                self.diagnostics.push(Diagnostic::new(
                    "E1199",
                    Severity::Error,
                    "parser recovery made no progress inside function body",
                    self.peek().span,
                ));
                self.bump();
            }
        }
        let end = self.expect(
            TokenKind::RBrace,
            "E1106",
            "expected `}` to close function body",
        );

        let mut tail_expr = None;
        if let Some(Stmt::ExprStmt { expr, .. }) = body.last() {
            tail_expr = Some(expr.clone());
            body.pop();
        }

        FunctionDecl {
            is_public: false,
            name,
            type_params,
            params,
            return_type,
            contracts,
            body,
            tail_expr,
            span: Span::new(start.line_start, start.col_start, end.line_end, end.col_end),
        }
    }

    fn parse_fn_literal_expression(&mut self) -> Expr {
        let start = self.bump().span;
        self.expect(
            TokenKind::LParen,
            "E1110",
            "expected `(` after `fn` in function literal",
        );
        let params = self.parse_params();
        self.expect(
            TokenKind::RParen,
            "E1111",
            "expected `)` after closure parameters",
        );
        let return_type = if self.match_kind(&TokenKind::Arrow) {
            Some(self.parse_type_ref_until(&[TokenKind::LBrace]))
        } else {
            None
        };
        self.expect(
            TokenKind::LBrace,
            "E1112",
            "expected `{` to start function literal body",
        );
        let mut body = Vec::new();
        let mut seen_exec = false;
        self.consume_newlines();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            let before = self.idx;
            self.consume_newlines();
            if self.at(&TokenKind::RBrace) || self.is_eof() {
                break;
            }
            if self.at(&TokenKind::At) {
                self.diagnostics.push(Diagnostic::new(
                    "E1310",
                    Severity::Error,
                    "closures cannot carry `@` contracts",
                    self.peek().span,
                ));
                let _ = self.parse_contract();
                self.consume_newlines();
                continue;
            }
            if !seen_exec {
                seen_exec = true;
            }
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            } else {
                self.sync_to_stmt_boundary();
            }
            self.consume_newlines();
            if self.idx == before && !self.at(&TokenKind::RBrace) && !self.is_eof() {
                self.diagnostics.push(Diagnostic::new(
                    "E1199",
                    Severity::Error,
                    "parser recovery made no progress inside function literal body",
                    self.peek().span,
                ));
                self.bump();
            }
        }
        let end = self.expect(
            TokenKind::RBrace,
            "E1113",
            "expected `}` to close function literal body",
        );
        let mut tail_expr = None;
        if let Some(Stmt::ExprStmt { expr, .. }) = body.last() {
            tail_expr = Some(Box::new(expr.clone()));
            body.pop();
        }
        let span = Span::new(start.line_start, start.col_start, end.line_end, end.col_end);
        Expr::FnLiteral {
            params,
            return_type,
            body,
            tail_expr,
            span,
        }
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        self.consume_newlines();
        while !self.at(&TokenKind::RParen) && !self.is_eof() {
            let is_mut = self.at_keyword(Keyword::Mut);
            if is_mut {
                self.bump();
            }
            let span = self.peek().span;
            let name = self.expect_ident("E1107", "expected parameter name");
            let ty = if self.match_kind(&TokenKind::Colon) {
                Some(self.parse_type_ref_until(&[TokenKind::Comma, TokenKind::RParen]))
            } else {
                None
            };
            params.push(Param {
                name,
                ty,
                is_mut,
                span,
            });
            if self.at(&TokenKind::Comma) {
                self.bump();
                self.consume_newlines();
            } else {
                break;
            }
        }
        params
    }

    fn parse_type_ref_until(&mut self, stops: &[TokenKind]) -> TypeRef {
        let mut raw = String::new();
        let mut angle_depth = 0i32;
        let mut paren_depth = 0i32;
        while !self.is_eof() {
            if angle_depth == 0 && paren_depth == 0 && stops.iter().any(|s| self.at(s)) {
                break;
            }
            if angle_depth == 0 && paren_depth == 0 && self.at(&TokenKind::Newline) {
                break;
            }
            let tok = self.bump();
            match tok.kind {
                TokenKind::LParen => paren_depth += 1,
                TokenKind::RParen if paren_depth > 0 => paren_depth -= 1,
                TokenKind::Lt => angle_depth += 1,
                TokenKind::Gt if angle_depth > 0 => angle_depth -= 1,
                _ => {}
            }
            raw.push_str(&tok.lexeme);
            if !self.at(&TokenKind::Comma)
                && !self.at(&TokenKind::RParen)
                && !self.at(&TokenKind::RBrace)
                && !self.at(&TokenKind::LBrace)
                && !self.at(&TokenKind::Newline)
            {
                raw.push(' ');
            }
        }
        TypeRef {
            raw: raw.trim().to_string(),
        }
    }

    fn parse_contract(&mut self) -> Contract {
        let at = self.expect(TokenKind::At, "E1301", "expected `@`");
        let name = self.expect_ident("E1302", "expected contract annotation name");
        match name.as_str() {
            "intent" => {
                if self.peek().kind == TokenKind::StringLit {
                    let tok = self.bump();
                    Contract::Intent {
                        text: tok.lexeme,
                        span: tok.span,
                    }
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "E1303",
                        Severity::Error,
                        "expected string literal after `@intent`",
                        self.peek().span,
                    ));
                    Contract::Intent {
                        text: String::new(),
                        span: at,
                    }
                }
            }
            "examples" => self.parse_examples_contract(at),
            "require" => Contract::Require {
                expr: self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]),
                span: at,
            },
            "ensure" => Contract::Ensure {
                expr: self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]),
                span: at,
            },
            "effect" => {
                let effect = self.expect_ident("E1304", "expected effect name after `@effect`");
                Contract::Effect {
                    name: effect,
                    span: at,
                }
            }
            "native" => {
                self.expect(TokenKind::LParen, "E1306", "expected `(` after `@native`");
                if self.peek().kind == TokenKind::StringLit {
                    let tok = self.bump();
                    self.expect(TokenKind::RParen, "E1306", "expected `)` after symbol name");
                    Contract::Native {
                        symbol: tok.lexeme,
                        span: at,
                    }
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "E1306",
                        Severity::Error,
                        "expected symbol name string after `@native(`",
                        self.peek().span,
                    ));
                    if self.peek().kind == TokenKind::RParen {
                        self.bump();
                    }
                    Contract::Native {
                        symbol: String::new(),
                        span: at,
                    }
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "E1305",
                    Severity::Error,
                    format!("unknown contract annotation `@{name}`"),
                    at,
                ));
                self.consume_line_end();
                Contract::Intent {
                    text: String::new(),
                    span: at,
                }
            }
        }
    }

    fn parse_examples_contract(&mut self, span: Span) -> Contract {
        self.expect(TokenKind::LBrace, "E1306", "expected `{` after `@examples`");
        let mut cases = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            let before = self.idx;
            self.consume_newlines();
            if self.at(&TokenKind::RBrace) || self.is_eof() {
                break;
            }
            let call = self.parse_expr_until(&[
                StopToken::FatArrow,
                StopToken::Newline,
                StopToken::RBrace,
            ]);
            if !self.match_kind(&TokenKind::FatArrow) {
                self.diagnostics.push(Diagnostic::new(
                    "E1307",
                    Severity::Error,
                    "expected `=>` in example case",
                    self.peek().span,
                ));
                self.sync_to_stmt_boundary();
                if self.at(&TokenKind::RBrace) {
                    break;
                }
                self.consume_newlines();
                continue;
            }
            let expected = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
            let case_span = Span::new(
                call.span().line_start,
                call.span().col_start,
                expected.span().line_end,
                expected.span().col_end,
            );
            cases.push(ExampleCase {
                call,
                expected,
                span: case_span,
            });
            self.consume_newlines();
            if self.idx == before && !self.at(&TokenKind::RBrace) && !self.is_eof() {
                self.diagnostics.push(Diagnostic::new(
                    "E1399",
                    Severity::Error,
                    "parser recovery made no progress inside `@examples`",
                    self.peek().span,
                ));
                self.bump();
            }
        }
        self.expect(
            TokenKind::RBrace,
            "E1308",
            "expected `}` to close `@examples`",
        );
        Contract::Examples { cases, span }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        self.consume_newlines();
        let start = self.peek().span;
        if self.at_keyword(Keyword::Return) {
            self.bump();
            let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
            return Some(Stmt::Return { expr, span: start });
        }
        if self.at_keyword(Keyword::Break) {
            self.bump();
            return Some(Stmt::Break { span: start });
        }
        if self.at_keyword(Keyword::Continue) {
            self.bump();
            return Some(Stmt::Continue { span: start });
        }
        if self.at_keyword(Keyword::For) {
            return Some(self.parse_for_stmt(start));
        }
        if self.at_keyword(Keyword::If) {
            return Some(self.parse_if_stmt(start));
        }
        if self.at_keyword(Keyword::While) {
            return Some(self.parse_while_stmt(start));
        }
        if self.at_keyword(Keyword::Repeat) {
            return Some(self.parse_repeat_stmt(start));
        }
        if self.at_keyword(Keyword::Select) {
            return Some(self.parse_select_stmt(start));
        }
        if self.at_keyword(Keyword::Match) {
            return Some(self.parse_match_stmt(start));
        }
        if self.at_keyword(Keyword::Go) {
            self.bump();
            let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
            return Some(Stmt::Go { expr, span: start });
        }
        if self.at_keyword(Keyword::Thread) {
            self.bump();
            let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
            return Some(Stmt::Thread { expr, span: start });
        }

        if self.at_malformed_mut() {
            // `mut name: T := expr` is the one malformed shape worth naming:
            // it is a type-annotated local, which the grammar does not accept
            // yet (`docs/spec/mutability_model.md` marks it TARGET), and the
            // generic message would send the reader looking for a typo.
            let message = if self.peek_n_kind(1) == Some(&TokenKind::Ident)
                && self.peek_n_kind(2) == Some(&TokenKind::Colon)
            {
                "type-annotated local bindings are not supported yet: write \
                 `mut name := expr` and annotate the value instead"
            } else {
                "`mut` must introduce a binding: write `mut name := expr`"
            };
            self.diagnostics
                .push(Diagnostic::new("E1213", Severity::Error, message, start));
            self.bump(); // `mut`, so recovery makes progress
            return None;
        }
        if let Some(stmt) = self.try_parse_binding(start) {
            return Some(stmt);
        }
        if let Some(stmt) = self.try_parse_assignment(start) {
            return Some(stmt);
        }

        let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
        Some(Stmt::ExprStmt { expr, span: start })
    }

    /// `true` when the statement starts with `mut` but is not `mut name := …`.
    /// Reported by [`Self::parse_stmt`] as `E1213` so the caller can resync,
    /// instead of leaking a chain of expression errors.
    fn at_malformed_mut(&self) -> bool {
        self.at_keyword(Keyword::Mut)
            && !(self.peek_n_kind(1) == Some(&TokenKind::Ident)
                && self.peek_n_kind(2) == Some(&TokenKind::Bind))
    }

    fn try_parse_binding(&mut self, span: Span) -> Option<Stmt> {
        if self.at_keyword(Keyword::Mut) {
            // `parse_stmt` reports the malformed shape as E1213 before getting
            // here; bail rather than bump past tokens on the strength of that.
            if self.at_malformed_mut() {
                return None;
            }
            self.bump(); // `mut`
            let name = self.bump().lexeme;
            self.bump(); // :=
            let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
            return Some(Stmt::Binding {
                name,
                is_mut: true,
                expr,
                span,
            });
        }
        if !(self.at_ident() && self.peek_n_kind(1) == Some(&TokenKind::Bind)) {
            return None;
        }
        let name = self.bump().lexeme;
        self.bump(); // :=
        let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
        Some(Stmt::Binding {
            name,
            is_mut: false,
            expr,
            span,
        })
    }

    fn try_parse_assignment(&mut self, span: Span) -> Option<Stmt> {
        let checkpoint = self.idx;
        let target = self.parse_lhs_expr()?;
        if !self.match_kind(&TokenKind::Assign) {
            self.idx = checkpoint;
            return None;
        }
        let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
        Some(Stmt::Assignment { target, expr, span })
    }

    fn parse_lhs_expr(&mut self) -> Option<Expr> {
        if !self.at_ident() {
            return None;
        }
        let tok = self.bump();
        let mut expr = Expr::Ident {
            name: tok.lexeme,
            span: tok.span,
        };
        while self.match_kind(&TokenKind::Dot) {
            if !self.at_ident() {
                break;
            }
            let field = self.bump();
            let sp = Span::new(
                expr.span().line_start,
                expr.span().col_start,
                field.span.line_end,
                field.span.col_end,
            );
            expr = Expr::Member {
                object: Box::new(expr),
                field: field.lexeme,
                span: sp,
            };
        }
        Some(expr)
    }

    fn parse_for_stmt(&mut self, span: Span) -> Stmt {
        self.bump(); // for
        let var = self.expect_ident("E1201", "expected loop variable in `for`");
        if !self.at_keyword(Keyword::In) {
            self.diagnostics.push(Diagnostic::new(
                "E1202",
                Severity::Error,
                "expected `in` in `for` statement",
                self.peek().span,
            ));
        } else {
            self.bump();
        }
        let iter = self.parse_expr_until(&[StopToken::LBrace]);
        let body = self.parse_block_body();
        Stmt::For {
            var,
            iter,
            body,
            span,
        }
    }

    fn parse_if_stmt(&mut self, span: Span) -> Stmt {
        self.bump(); // if
        let cond = self.parse_expr_until(&[StopToken::LBrace]);
        let then_body = self.parse_block_body();
        let else_body = if self.at_keyword(Keyword::Else) {
            self.bump();
            self.parse_block_body()
        } else {
            Vec::new()
        };
        Stmt::If {
            cond,
            then_body,
            else_body,
            span,
        }
    }

    fn parse_while_stmt(&mut self, span: Span) -> Stmt {
        self.bump(); // while
        let cond = self.parse_expr_until(&[StopToken::LBrace]);
        let body = self.parse_block_body();
        Stmt::While { cond, body, span }
    }

    fn parse_repeat_stmt(&mut self, span: Span) -> Stmt {
        self.bump(); // repeat
        let count = self.parse_expr_until(&[StopToken::LBrace]);
        let body = self.parse_block_body();
        Stmt::Repeat { count, body, span }
    }

    fn parse_select_stmt(&mut self, span: Span) -> Stmt {
        self.bump(); // select
        self.expect(TokenKind::LBrace, "E1203", "expected `{` after `select`");
        let mut cases = Vec::new();
        self.consume_newlines();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            let before = self.idx;
            self.consume_newlines();
            if !self.at_keyword(Keyword::Case) {
                self.diagnostics.push(Diagnostic::new(
                    "E1204",
                    Severity::Error,
                    "expected `case` in `select` block",
                    self.peek().span,
                ));
                self.sync_to_stmt_boundary();
                continue;
            }
            self.bump();
            let pattern = self.parse_select_pattern();
            self.expect(
                TokenKind::FatArrow,
                "E1205",
                "expected `=>` in `select` case",
            );
            let action = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
            cases.push(SelectCase {
                pattern,
                action,
                span,
            });
            self.consume_newlines();
            if self.idx == before && !self.at(&TokenKind::RBrace) && !self.is_eof() {
                self.diagnostics.push(Diagnostic::new(
                    "E1299",
                    Severity::Error,
                    "parser recovery made no progress inside `select`",
                    self.peek().span,
                ));
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace, "E1206", "expected `}` to close `select`");
        Stmt::Select { cases, span }
    }

    fn parse_match_stmt(&mut self, span: Span) -> Stmt {
        self.bump(); // match
        let scrutinee = self.parse_expr_until(&[StopToken::LBrace]);
        self.expect(
            TokenKind::LBrace,
            "E1203",
            "expected `{` after `match` scrutinee",
        );
        let mut arms = Vec::new();
        let mut default_action = None;
        self.consume_newlines();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            self.consume_newlines();
            if self.at(&TokenKind::RBrace) {
                break;
            }
            if self.at_keyword(Keyword::Default) {
                self.bump();
                self.expect(
                    TokenKind::FatArrow,
                    "E1205",
                    "expected `=>` after `default`",
                );
                default_action =
                    Some(self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]));
                self.consume_newlines();
                break;
            }
            if self.at_keyword(Keyword::Case) {
                self.bump();
                let pattern = self.parse_expr_until(&[
                    StopToken::FatArrow,
                    StopToken::Newline,
                    StopToken::RBrace,
                ]);
                self.expect(
                    TokenKind::FatArrow,
                    "E1205",
                    "expected `=>` in `match` case",
                );
                let action = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
                let arm_span = Span::new(
                    pattern.span().line_start,
                    pattern.span().col_start,
                    action.span().line_end,
                    action.span().col_end,
                );
                arms.push(MatchArm {
                    pattern,
                    action,
                    span: arm_span,
                });
            } else {
                self.sync_to_stmt_boundary();
            }
            self.consume_newlines();
        }
        self.expect(TokenKind::RBrace, "E1206", "expected `}` to close `match`");
        Stmt::Match {
            scrutinee,
            arms,
            default_action,
            span,
        }
    }

    fn parse_select_pattern(&mut self) -> SelectPattern {
        if self.at_keyword(Keyword::Default) {
            self.bump();
            return SelectPattern::Default;
        }
        if self.at_keyword(Keyword::After) {
            self.bump();
            let lit = self.bump().lexeme;
            return SelectPattern::After {
                duration_literal: lit,
            };
        }
        if self.at_keyword(Keyword::Closed) {
            self.bump();
            return SelectPattern::Closed {
                ident: self.expect_ident("E1207", "expected identifier after `closed`"),
            };
        }
        let binding = self.expect_ident("E1208", "expected binding identifier in `select` receive");
        self.expect(TokenKind::Bind, "E1209", "expected `:=` in receive case");
        let expr =
            self.parse_expr_until(&[StopToken::FatArrow, StopToken::Newline, StopToken::RBrace]);
        SelectPattern::Receive { binding, expr }
    }

    fn parse_block_body(&mut self) -> Vec<Stmt> {
        self.expect(TokenKind::LBrace, "E1210", "expected `{` to start block");
        let mut body = Vec::new();
        self.consume_newlines();
        while !self.at(&TokenKind::RBrace) && !self.is_eof() {
            let before = self.idx;
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            } else {
                self.sync_to_stmt_boundary();
            }
            self.consume_newlines();
            if self.idx == before && !self.at(&TokenKind::RBrace) && !self.is_eof() {
                self.diagnostics.push(Diagnostic::new(
                    "E1212",
                    Severity::Error,
                    "parser recovery made no progress inside block",
                    self.peek().span,
                ));
                self.bump();
            }
        }
        self.expect(TokenKind::RBrace, "E1211", "expected `}` to close block");
        body
    }

    fn parse_expr_until(&mut self, stop: &[StopToken]) -> Expr {
        self.parse_binary_expr(0, stop)
            .unwrap_or_else(|| self.error_expr("E1401", "expected expression"))
    }

    /// Depth-guarded entry point for expression recursion. Every nested
    /// `(`/`[`/`{` primary, call-argument list, and binary right-operand
    /// re-enters the parser through here, so a single counter bounds all
    /// of them and turns a would-be stack overflow into a clean E1415.
    fn parse_binary_expr(&mut self, min_prec: u8, stop: &[StopToken]) -> Option<Expr> {
        if self.expr_depth >= MAX_EXPR_NESTING_DEPTH {
            return Some(self.expr_nesting_too_deep());
        }
        self.expr_depth += 1;
        let result = self.parse_binary_expr_inner(min_prec, stop);
        self.expr_depth -= 1;
        if self.expr_depth == 0 {
            self.expr_depth_exceeded = false;
        }
        result
    }

    fn parse_binary_expr_inner(&mut self, min_prec: u8, stop: &[StopToken]) -> Option<Expr> {
        let mut left = self.parse_unary_expr(stop)?;
        loop {
            if self.is_stop(stop) {
                break;
            }
            let (op, prec) = match self.peek().kind {
                TokenKind::Star => (BinaryOp::Mul, 40),
                TokenKind::Slash => (BinaryOp::Div, 40),
                TokenKind::Percent => (BinaryOp::Mod, 40),
                TokenKind::LtLt => (BinaryOp::Shl, 35),
                TokenKind::GtGt => (BinaryOp::Shr, 35),
                TokenKind::Plus => (BinaryOp::Add, 30),
                TokenKind::Minus => (BinaryOp::Sub, 30),
                TokenKind::Lt => (BinaryOp::Lt, 25),
                TokenKind::Le => (BinaryOp::Le, 25),
                TokenKind::Gt => (BinaryOp::Gt, 25),
                TokenKind::Ge => (BinaryOp::Ge, 25),
                TokenKind::EqEq => (BinaryOp::Eq, 20),
                TokenKind::NotEq => (BinaryOp::Ne, 20),
                TokenKind::Amp => (BinaryOp::BitAnd, 18),
                TokenKind::Caret => (BinaryOp::BitXor, 16),
                TokenKind::Pipe => (BinaryOp::BitOr, 14),
                TokenKind::AmpAmp => (BinaryOp::And, 10),
                TokenKind::PipePipe => (BinaryOp::Or, 5),
                _ => break,
            };
            if prec < min_prec {
                break;
            }
            self.bump();
            let right = self
                .parse_binary_expr(prec + 1, stop)
                .unwrap_or_else(|| self.error_expr("E1402", "expected right-hand expression"));
            let span = Span::new(
                left.span().line_start,
                left.span().col_start,
                right.span().line_end,
                right.span().col_end,
            );
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Some(left)
    }

    fn parse_unary_expr(&mut self, stop: &[StopToken]) -> Option<Expr> {
        if self.is_stop(stop) {
            return None;
        }
        let expr = match self.peek().kind {
            TokenKind::Minus => {
                let op_tok = self.bump();
                let inner =
                    self.parse_unary_operand(stop, "E1403", "expected expression after unary `-`");
                let span = Span::new(
                    op_tok.span.line_start,
                    op_tok.span.col_start,
                    inner.span().line_end,
                    inner.span().col_end,
                );
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(inner),
                    span,
                }
            }
            TokenKind::Bang => {
                let op_tok = self.bump();
                let inner =
                    self.parse_unary_operand(stop, "E1404", "expected expression after unary `!`");
                let span = Span::new(
                    op_tok.span.line_start,
                    op_tok.span.col_start,
                    inner.span().line_end,
                    inner.span().col_end,
                );
                Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(inner),
                    span,
                }
            }
            TokenKind::Keyword(Keyword::Async) => {
                let kw = self.bump();
                let inner =
                    self.parse_unary_operand(stop, "E1404A", "expected expression after `async`");
                let span = Span::new(
                    kw.span.line_start,
                    kw.span.col_start,
                    inner.span().line_end,
                    inner.span().col_end,
                );
                Expr::Async {
                    expr: Box::new(inner),
                    span,
                }
            }
            TokenKind::Keyword(Keyword::Await) => {
                let kw = self.bump();
                let inner =
                    self.parse_unary_operand(stop, "E1404B", "expected expression after `await`");
                let span = Span::new(
                    kw.span.line_start,
                    kw.span.col_start,
                    inner.span().line_end,
                    inner.span().col_end,
                );
                Expr::Await {
                    expr: Box::new(inner),
                    span,
                }
            }
            _ => self.parse_postfix_expr(stop)?,
        };
        Some(expr)
    }

    /// Depth-guarded operand parse for the self-recursive unary arms
    /// (`-`, `!`, `async`, `await`), so unbounded operator chains hit the
    /// same E1415 guard as bracket nesting instead of overflowing the stack.
    fn parse_unary_operand(&mut self, stop: &[StopToken], code: &str, message: &str) -> Expr {
        if self.expr_depth >= MAX_EXPR_NESTING_DEPTH {
            return self.expr_nesting_too_deep();
        }
        self.expr_depth += 1;
        let inner = self.parse_unary_expr(stop);
        self.expr_depth -= 1;
        inner.unwrap_or_else(|| self.error_expr(code, message))
    }

    /// Reports E1415 once for the over-deep expression, then skips the
    /// remainder of that expression (balanced over `()[]{}`, stopping
    /// before the enclosing closer, a top-level newline/comma, or EOF) so
    /// the enclosing frames unwind without cascading follow-on errors.
    fn expr_nesting_too_deep(&mut self) -> Expr {
        let span = self.peek().span;
        self.diagnostics.push(Diagnostic::new(
            "E1415",
            Severity::Error,
            format!("expression nesting too deep (limit is {MAX_EXPR_NESTING_DEPTH})"),
            span,
        ));
        self.expr_depth_exceeded = true;
        let mut depth = 0usize;
        loop {
            let kind = self.peek().kind.clone();
            match kind {
                TokenKind::Eof => break,
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                TokenKind::Newline | TokenKind::Comma if depth == 0 => break,
                _ => {}
            }
            self.bump();
        }
        Expr::Ident {
            name: "__error".to_string(),
            span,
        }
    }

    fn parse_postfix_expr(&mut self, stop: &[StopToken]) -> Option<Expr> {
        let mut expr = self.parse_primary(stop)?;
        loop {
            if self.at(&TokenKind::Newline)
                && self
                    .peek_non_newline_kind()
                    .is_some_and(|k| k == TokenKind::Dot)
            {
                self.consume_newlines();
            }
            if self.is_stop(stop) {
                break;
            }
            let is_constructor = matches!(&expr, Expr::Ident { .. }) && self.at(&TokenKind::LBrace);
            if is_constructor {
                let (type_name, ident_span) = match &expr {
                    Expr::Ident { name, span } => (name.clone(), *span),
                    _ => unreachable!(),
                };
                self.bump(); // consume {
                let mut fields = Vec::new();
                self.consume_newlines();
                while !self.at(&TokenKind::RBrace) && !self.is_eof() {
                    let field_name =
                        self.expect_ident("E1140", "expected field name in constructor");
                    self.expect(
                        TokenKind::Colon,
                        "E1141",
                        "expected `:` after field name in constructor",
                    );
                    let value = self.parse_expr_until(&[StopToken::Comma, StopToken::RBrace]);
                    fields.push((field_name, value));
                    if self.match_kind(&TokenKind::Comma) {
                        self.consume_newlines();
                    } else {
                        break;
                    }
                }
                let end = self.expect(
                    TokenKind::RBrace,
                    "E1412",
                    "expected `}` to close constructor",
                );
                expr = Expr::Constructor {
                    type_name,
                    fields,
                    span: Span::new(
                        ident_span.line_start,
                        ident_span.col_start,
                        end.line_end,
                        end.col_end,
                    ),
                };
                continue;
            }
            let is_enum_variant_ctor = matches!(
                &expr,
                Expr::Member { object, .. } if matches!(&**object, Expr::Ident { .. })
            ) && self.at(&TokenKind::LBrace);
            if is_enum_variant_ctor {
                let (enum_name, variant, span_lo) = match &expr {
                    Expr::Member {
                        object,
                        field,
                        span,
                    } => {
                        let Expr::Ident { name, .. } = &**object else {
                            unreachable!()
                        };
                        (name.clone(), field.clone(), *span)
                    }
                    _ => unreachable!(),
                };
                self.bump(); // `{`
                let mut fields = Vec::new();
                self.consume_newlines();
                while !self.at(&TokenKind::RBrace) && !self.is_eof() {
                    if !self.at_ident() {
                        self.diagnostics.push(Diagnostic::new(
                            "E1140",
                            Severity::Error,
                            "expected field name in enum variant expression",
                            self.peek().span,
                        ));
                        break;
                    }
                    let ftok = self.bump();
                    let field_name = ftok.lexeme;
                    let field_span = ftok.span;
                    if self.at(&TokenKind::Comma) || self.at(&TokenKind::RBrace) {
                        fields.push((
                            field_name.clone(),
                            Expr::Ident {
                                name: field_name,
                                span: field_span,
                            },
                        ));
                    } else {
                        self.expect(
                            TokenKind::Colon,
                            "E1141",
                            "expected `:` after field name in enum variant expression",
                        );
                        let value = self.parse_expr_until(&[StopToken::Comma, StopToken::RBrace]);
                        fields.push((field_name, value));
                    }
                    if self.match_kind(&TokenKind::Comma) {
                        self.consume_newlines();
                    } else {
                        break;
                    }
                }
                let end = self.expect(
                    TokenKind::RBrace,
                    "E1412",
                    "expected `}` to close enum variant expression",
                );
                expr = Expr::EnumVariant {
                    enum_name,
                    variant,
                    fields,
                    span: Span::new(
                        span_lo.line_start,
                        span_lo.col_start,
                        end.line_end,
                        end.col_end,
                    ),
                };
                continue;
            }
            if self.match_kind(&TokenKind::Dot) {
                if !self.at_ident() {
                    self.diagnostics.push(Diagnostic::new(
                        "E1405",
                        Severity::Error,
                        "expected identifier after `.`",
                        self.peek().span,
                    ));
                    break;
                }
                let field = self.bump();
                let span = Span::new(
                    expr.span().line_start,
                    expr.span().col_start,
                    field.span.line_end,
                    field.span.col_end,
                );
                expr = Expr::Member {
                    object: Box::new(expr),
                    field: field.lexeme,
                    span,
                };
                continue;
            }
            if self.match_kind(&TokenKind::LParen) {
                let mut args = Vec::new();
                self.consume_newlines();
                while !self.at(&TokenKind::RParen) && !self.is_eof() {
                    args.push(self.parse_expr_until(&[
                        StopToken::Comma,
                        StopToken::RParen,
                        StopToken::Newline,
                    ]));
                    if self.match_kind(&TokenKind::Comma) {
                        self.consume_newlines();
                    } else {
                        break;
                    }
                }
                let end = self.expect(
                    TokenKind::RParen,
                    "E1406",
                    "expected `)` after call arguments",
                );
                let span = Span::new(
                    expr.span().line_start,
                    expr.span().col_start,
                    end.line_end,
                    end.col_end,
                );
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
                continue;
            }
            if self.match_kind(&TokenKind::LBracket) {
                self.consume_newlines();
                if self.match_kind(&TokenKind::Colon) {
                    self.consume_newlines();
                    let end = if self.at(&TokenKind::RBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expr_until(&[StopToken::RBracket])))
                    };
                    let close = self.expect(TokenKind::RBracket, "E1406A", "expected `]`");
                    let span = Span::new(
                        expr.span().line_start,
                        expr.span().col_start,
                        close.line_end,
                        close.col_end,
                    );
                    expr = Expr::Slice {
                        object: Box::new(expr),
                        start: None,
                        end,
                        span,
                    };
                    continue;
                }
                let first = self.parse_expr_until(&[StopToken::Colon, StopToken::RBracket]);
                if self.match_kind(&TokenKind::Colon) {
                    self.consume_newlines();
                    let end = if self.at(&TokenKind::RBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expr_until(&[StopToken::RBracket])))
                    };
                    let close = self.expect(TokenKind::RBracket, "E1406B", "expected `]`");
                    let span = Span::new(
                        expr.span().line_start,
                        expr.span().col_start,
                        close.line_end,
                        close.col_end,
                    );
                    expr = Expr::Slice {
                        object: Box::new(expr),
                        start: Some(Box::new(first)),
                        end,
                        span,
                    };
                    continue;
                }
                let close = self.expect(TokenKind::RBracket, "E1406C", "expected `]`");
                let span = Span::new(
                    expr.span().line_start,
                    expr.span().col_start,
                    close.line_end,
                    close.col_end,
                );
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(first),
                    span,
                };
                continue;
            }
            if self.match_kind(&TokenKind::Question) {
                let sp = expr.span();
                expr = Expr::Question {
                    expr: Box::new(expr),
                    span: sp,
                };
                continue;
            }
            break;
        }
        Some(expr)
    }

    fn parse_primary(&mut self, stop: &[StopToken]) -> Option<Expr> {
        if self.is_stop(stop) {
            return None;
        }
        // `mut` in expression position — the call-site borrow form `f(mut x)`
        // that other languages have and VibeLang does not. Report it once and
        // keep parsing the operand, so the caller sees `f(x)` and gets exactly
        // one diagnostic instead of a cascade of expression/`)` errors.
        if self.at_keyword(Keyword::Mut) {
            self.diagnostics.push(Diagnostic::new(
                "E1213",
                Severity::Error,
                "`mut` is not valid here: mutability is declared at the binding \
                 (`mut name := expr`) or on the parameter (`fn f(mut name: T)`), \
                 never at a call site",
                self.peek().span,
            ));
            self.bump();
            if self.is_stop(stop) {
                return None;
            }
        }
        let tok = self.peek().clone();
        let expr = match &tok.kind {
            TokenKind::Ident => {
                let ident = self.bump();
                if ident.lexeme == "old" && self.at(&TokenKind::LParen) {
                    self.bump();
                    let inner = self.parse_expr_until(&[StopToken::RParen]);
                    let end = self.expect(
                        TokenKind::RParen,
                        "E1407",
                        "expected `)` after `old(` expression",
                    );
                    let span = Span::new(
                        ident.span.line_start,
                        ident.span.col_start,
                        end.line_end,
                        end.col_end,
                    );
                    Expr::Old {
                        expr: Box::new(inner),
                        span,
                    }
                } else {
                    Expr::Ident {
                        name: ident.lexeme,
                        span: ident.span,
                    }
                }
            }
            TokenKind::IntLit => {
                let t = self.bump();
                // Int literal lexemes are pure digit runs, so parse failure
                // means the value is out of `i64` range. Note `i64::MIN`
                // written as `-9223372036854775808` also errors here: the
                // unary minus applies to a `9223372036854775808` literal,
                // which is itself out of range (same behavior as rustc).
                let value = match t.lexeme.parse() {
                    Ok(value) => value,
                    Err(_) => {
                        self.diagnostics.push(Diagnostic::new(
                            "E1414",
                            Severity::Error,
                            "integer literal out of range for Int",
                            t.span,
                        ));
                        0
                    }
                };
                Expr::Int {
                    value,
                    span: t.span,
                }
            }
            TokenKind::FloatLit => {
                let t = self.bump();
                Expr::Float {
                    value: t.lexeme.parse().unwrap_or_default(),
                    span: t.span,
                }
            }
            TokenKind::StringLit => {
                let t = self.bump();
                if let Some(interp) = self.try_desugar_string_interp(&t.lexeme, t.span) {
                    interp
                } else {
                    let cleaned = t.lexeme.replace('\x00', "");
                    Expr::String {
                        value: cleaned,
                        span: t.span,
                    }
                }
            }
            TokenKind::Keyword(Keyword::True) => {
                let t = self.bump();
                Expr::Bool {
                    value: true,
                    span: t.span,
                }
            }
            TokenKind::Keyword(Keyword::False) => {
                let t = self.bump();
                Expr::Bool {
                    value: false,
                    span: t.span,
                }
            }
            TokenKind::Keyword(Keyword::Fn) => self.parse_fn_literal_expression(),
            TokenKind::LBracket => {
                let start = self.bump().span;
                let mut items = Vec::new();
                self.consume_newlines();
                while !self.at(&TokenKind::RBracket) && !self.is_eof() {
                    items.push(self.parse_expr_until(&[StopToken::Comma, StopToken::RBracket]));
                    if self.match_kind(&TokenKind::Comma) {
                        self.consume_newlines();
                    } else {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RBracket, "E1408", "expected `]`");
                let span = Span::new(start.line_start, start.col_start, end.line_end, end.col_end);
                Expr::List { items, span }
            }
            TokenKind::LBrace => {
                let start = self.bump().span;
                let mut entries = Vec::new();
                self.consume_newlines();
                while !self.at(&TokenKind::RBrace) && !self.is_eof() {
                    let key = self.parse_expr_until(&[StopToken::Colon, StopToken::RBrace]);
                    if !self.match_kind(&TokenKind::Colon) {
                        // Suppressed while unwinding from an E1415 overflow.
                        if !self.expr_depth_exceeded {
                            self.diagnostics.push(Diagnostic::new(
                                "E1411",
                                Severity::Error,
                                "expected `:` after map key",
                                self.peek().span,
                            ));
                        }
                        break;
                    }
                    let value = self.parse_expr_until(&[StopToken::Comma, StopToken::RBrace]);
                    entries.push((key, value));
                    if self.match_kind(&TokenKind::Comma) {
                        self.consume_newlines();
                    } else {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RBrace, "E1412", "expected `}` after map literal");
                let span = Span::new(start.line_start, start.col_start, end.line_end, end.col_end);
                Expr::Map { entries, span }
            }
            TokenKind::Dot => {
                let t = self.bump();
                Expr::DotResult { span: t.span }
            }
            TokenKind::LParen => {
                self.bump();
                let inner = self.parse_expr_until(&[StopToken::RParen]);
                self.expect(TokenKind::RParen, "E1409", "expected `)`");
                inner
            }
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "E1410",
                    Severity::Error,
                    "unexpected token in expression",
                    tok.span,
                ));
                return None;
            }
        };
        Some(expr)
    }

    fn try_desugar_string_interp(&mut self, raw: &str, span: Span) -> Option<Expr> {
        let chars: Vec<char> = raw.chars().collect();
        let len = chars.len();

        let mut has_interp = false;
        let mut i = 0;
        while i < len {
            if chars[i] == '\x00' {
                i += 2;
                continue;
            }
            if chars[i] == '{'
                && i + 1 < len
                && (chars[i + 1].is_ascii_alphabetic() || chars[i + 1] == '_')
            {
                has_interp = true;
                break;
            }
            i += 1;
        }
        if !has_interp {
            return None;
        }

        enum Part {
            Text(String),
            Ident(String),
        }

        let mut parts: Vec<Part> = Vec::new();
        let mut current_text = String::new();
        let mut i = 0;

        while i < len {
            if chars[i] == '\x00' && i + 1 < len {
                current_text.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == '{'
                && i + 1 < len
                && (chars[i + 1].is_ascii_alphabetic() || chars[i + 1] == '_')
            {
                if !current_text.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut current_text)));
                }
                i += 1;
                let mut ident = String::new();
                while i < len && chars[i] != '}' {
                    ident.push(chars[i]);
                    i += 1;
                }
                if i < len {
                    i += 1; // skip '}'
                }
                let ident = ident.trim().to_string();
                if ident.is_empty() {
                    self.diagnostics.push(Diagnostic::new(
                        "E1413",
                        Severity::Error,
                        "empty interpolation expression `{}`",
                        span,
                    ));
                } else {
                    parts.push(Part::Ident(ident));
                }
            } else {
                current_text.push(chars[i]);
                i += 1;
            }
        }
        if !current_text.is_empty() {
            parts.push(Part::Text(current_text));
        }

        if parts.is_empty() {
            return Some(Expr::String {
                value: String::new(),
                span,
            });
        }

        fn text_expr(s: String, span: Span) -> Expr {
            Expr::String { value: s, span }
        }

        fn ident_to_str_call(ident_name: String, span: Span) -> Expr {
            Expr::Call {
                callee: Box::new(Expr::Member {
                    object: Box::new(Expr::Ident {
                        name: "convert".to_string(),
                        span,
                    }),
                    field: "to_str".to_string(),
                    span,
                }),
                args: vec![Expr::Ident {
                    name: ident_name,
                    span,
                }],
                span,
            }
        }

        fn concat(left: Expr, right: Expr, span: Span) -> Expr {
            Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
                span,
            }
        }

        let mut result: Option<Expr> = None;
        for part in parts {
            let expr = match part {
                Part::Text(s) => text_expr(s, span),
                Part::Ident(name) => ident_to_str_call(name, span),
            };
            result = Some(match result {
                None => expr,
                Some(acc) => concat(acc, expr, span),
            });
        }

        result
    }

    fn error_expr(&mut self, code: &str, message: &str) -> Expr {
        let span = self.peek().span;
        // Suppressed while unwinding from an E1415 nesting overflow.
        if !self.expr_depth_exceeded {
            self.diagnostics
                .push(Diagnostic::new(code, Severity::Error, message, span));
        }
        Expr::Ident {
            name: "__error".to_string(),
            span,
        }
    }

    fn parse_qualified_ident(&mut self) -> String {
        let mut name = self.expect_ident("E1150", "expected identifier");
        while self.match_kind(&TokenKind::Dot) {
            let part = self.expect_ident("E1151", "expected identifier after `.`");
            name.push('.');
            name.push_str(&part);
        }
        name
    }

    fn sync_to_decl(&mut self) {
        while !self.is_eof() {
            if self.at_keyword(Keyword::Pub) || self.at_ident() {
                return;
            }
            self.bump();
        }
    }

    fn sync_to_stmt_boundary(&mut self) {
        while !self.is_eof() {
            if self.at(&TokenKind::Newline) {
                self.consume_newlines();
                return;
            }
            if self.at(&TokenKind::RBrace) {
                return;
            }
            self.bump();
        }
    }

    fn consume_newlines(&mut self) {
        while self.at(&TokenKind::Newline) {
            self.bump();
        }
    }

    fn consume_line_end(&mut self) {
        if self.at(&TokenKind::Newline) {
            self.consume_newlines();
        }
    }

    fn is_stop(&self, stop: &[StopToken]) -> bool {
        stop.iter().any(|s| match s {
            StopToken::Newline => {
                if self.at(&TokenKind::Newline)
                    && self
                        .peek_non_newline_kind()
                        .is_some_and(|k| k == TokenKind::Dot)
                {
                    return false;
                }
                self.at(&TokenKind::Newline)
            }
            StopToken::Comma => self.at(&TokenKind::Comma),
            StopToken::Colon => self.at(&TokenKind::Colon),
            StopToken::RParen => self.at(&TokenKind::RParen),
            StopToken::RBrace => self.at(&TokenKind::RBrace),
            StopToken::RBracket => self.at(&TokenKind::RBracket),
            StopToken::FatArrow => self.at(&TokenKind::FatArrow),
            StopToken::LBrace => self.at(&TokenKind::LBrace),
        })
    }

    fn peek_non_newline_kind(&self) -> Option<TokenKind> {
        let mut i = self.idx;
        while let Some(tok) = self.tokens.get(i) {
            if tok.kind != TokenKind::Newline {
                return Some(tok.kind.clone());
            }
            i += 1;
        }
        None
    }

    fn at_ident(&self) -> bool {
        self.peek().kind == TokenKind::Ident
    }

    fn at_keyword(&self, kw: Keyword) -> bool {
        self.peek().kind == TokenKind::Keyword(kw)
    }

    fn at(&self, kind: &TokenKind) -> bool {
        self.peek().kind == *kind
    }

    fn expect_ident(&mut self, code: &str, message: &str) -> String {
        if self.at_ident() {
            self.bump().lexeme
        } else {
            let span = self.peek().span;
            self.diagnostics
                .push(Diagnostic::new(code, Severity::Error, message, span));
            "__error".to_string()
        }
    }

    fn expect(&mut self, kind: TokenKind, code: &str, message: &str) -> Span {
        if self.at(&kind) {
            self.bump().span
        } else {
            let span = self.peek().span;
            // While unwinding from an expression-nesting overflow, the
            // abandoned frames would each report a missing token here;
            // the E1415 already explains the failure, so stay quiet.
            if !self.expr_depth_exceeded {
                self.diagnostics
                    .push(Diagnostic::new(code, Severity::Error, message, span));
            }
            span
        }
    }

    fn match_kind(&mut self, kind: &TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn peek_n_kind(&self, n: usize) -> Option<&TokenKind> {
        self.tokens.get(self.idx + n).map(|t| &t.kind)
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.idx)
            .unwrap_or_else(|| self.tokens.last().expect("lexer emits EOF token"))
    }

    fn bump(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.idx < self.tokens.len().saturating_sub(1) {
            self.idx += 1;
        }
        tok
    }

    fn is_eof(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

#[cfg(test)]
mod tests {
    use super::parse_source;
    use vibe_ast::{Declaration, Expr, Stmt};
    use vibe_diagnostics::Severity;

    #[test]
    fn parses_basic_function() {
        let src = r#"
topK(xs, k) {
  @intent "k largest"
  @ensure len(.) > 0
  xs.sort_desc().take(k)
}
"#;
        let out = parse_source(src);
        assert!(!out.ast.declarations.is_empty());
        assert!(
            !out.diagnostics.has_errors(),
            "{}",
            out.diagnostics.to_golden()
        );
    }

    #[test]
    fn parses_generic_function_decl() {
        let src = r#"identity<T>(x: T) -> T { x }"#;
        let out = parse_source(src);
        assert!(
            !out.diagnostics.has_errors(),
            "{}",
            out.diagnostics.to_golden()
        );
        match &out.ast.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.type_params, vec!["T".to_string()]);
                assert_eq!(f.name, "identity");
            }
            _ => panic!("expected function decl"),
        }
    }

    #[test]
    fn parses_break_and_continue_statements() {
        let src = r#"
main() -> Int {
  i := 0
  while i < 10 {
    i = i + 1
    if i == 3 {
      continue
    } else {
      if i == 8 {
        break
      }
    }
  }
  return i
}
"#;
        let out = parse_source(src);
        assert!(
            !out.diagnostics.has_errors(),
            "{}",
            out.diagnostics.to_golden()
        );
    }

    #[test]
    fn parses_i64_max_int_literal() {
        let src = "main() -> Int {\n  x := 9223372036854775807\n  x\n}\n";
        let out = parse_source(src);
        assert!(
            !out.diagnostics.has_errors(),
            "{}",
            out.diagnostics.to_golden()
        );
        let func = match &out.ast.declarations[0] {
            Declaration::Function(f) => f,
            _ => panic!("expected function decl"),
        };
        let expr = match &func.body[0] {
            Stmt::Binding { expr, .. } => expr,
            _ => panic!("expected binding statement"),
        };
        match expr {
            Expr::Int { value, .. } => assert_eq!(*value, i64::MAX),
            _ => panic!("expected int literal"),
        }
    }

    #[test]
    fn rejects_int_literal_above_i64_max() {
        let src = "main() -> Int {\n  x := 9223372036854775808\n  0\n}\n";
        let out = parse_source(src);
        let errors: Vec<_> = out
            .diagnostics
            .as_slice()
            .iter()
            .filter(|d| d.code == "E1414")
            .collect();
        assert_eq!(errors.len(), 1, "{}", out.diagnostics.to_golden());
        let diag = errors[0];
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "integer literal out of range for Int");
        assert_eq!(
            (
                diag.span.line_start,
                diag.span.col_start,
                diag.span.line_end,
                diag.span.col_end,
            ),
            (2, 8, 2, 26),
            "diagnostic should span the literal token"
        );
    }

    #[test]
    fn rejects_i64_min_written_as_negative_literal() {
        // `-9223372036854775808` is unary minus applied to a
        // `9223372036854775808` literal, which is out of range on its own
        // (same behavior as rustc). Spell i64::MIN arithmetically instead.
        let src = "main() -> Int {\n  x := -9223372036854775808\n  0\n}\n";
        let out = parse_source(src);
        assert!(
            out.diagnostics.as_slice().iter().any(|d| d.code == "E1414"),
            "{}",
            out.diagnostics.to_golden()
        );
    }

    #[test]
    fn rejects_thirty_digit_int_literal() {
        let src = "main() -> Int {\n  x := 999999999999999999999999999999\n  0\n}\n";
        let out = parse_source(src);
        let errors: Vec<_> = out
            .diagnostics
            .as_slice()
            .iter()
            .filter(|d| d.code == "E1414")
            .collect();
        assert_eq!(errors.len(), 1, "{}", out.diagnostics.to_golden());
    }

    /// `main() { v := <open*n><atom><close*n> ... }` — one bracket level
    /// per `n`, used to probe the expression nesting-depth guard.
    fn nested_expr_src(open: &str, close: &str, atom: &str, n: usize) -> String {
        format!(
            "main() {{\n  v := {}{}{}\n  v\n}}\n",
            open.repeat(n),
            atom,
            close.repeat(n)
        )
    }

    /// The three bracketed shapes that recurse per nesting level:
    /// parens, list literals, map literals.
    fn nesting_shapes() -> [(&'static str, &'static str, &'static str); 3] {
        [("(", ")", "1"), ("[", "]", "1"), ("{1: ", "}", "0")]
    }

    /// Runs a deep-nesting test on a thread with a large explicit stack.
    /// Near the limit the parser holds up to `MAX_EXPR_NESTING_DEPTH`
    /// frames (~15.6 KiB each in debug, ~3.9 MiB total), which fits the
    /// 8 MiB main thread the CLI and LSP parse on but not the 2 MiB
    /// default stack of Rust test threads in debug builds.
    fn on_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(f)
            .expect("spawn deep-nesting test thread")
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    }

    #[test]
    fn accepts_expression_nesting_below_limit() {
        on_big_stack(|| {
            for (open, close, atom) in nesting_shapes() {
                let src = nested_expr_src(open, close, atom, super::MAX_EXPR_NESTING_DEPTH - 1);
                let out = parse_source(&src);
                assert!(
                    !out.diagnostics.has_errors(),
                    "shape {open}...{close} at limit-1 should parse cleanly:\n{}",
                    out.diagnostics.to_golden()
                );
            }
        });
    }

    #[test]
    fn rejects_expression_nesting_at_limit_with_single_error() {
        on_big_stack(|| {
            for (open, close, atom) in nesting_shapes() {
                let src = nested_expr_src(open, close, atom, super::MAX_EXPR_NESTING_DEPTH);
                let out = parse_source(&src);
                let diags = out.diagnostics.as_slice();
                assert_eq!(
                    diags.len(),
                    1,
                    "shape {open}...{close} at limit should report exactly one diagnostic:\n{}",
                    out.diagnostics.to_golden()
                );
                assert_eq!(diags[0].code, "E1415");
                assert_eq!(diags[0].severity, Severity::Error);
                assert!(diags[0].message.contains("expression nesting too deep"));
                assert_eq!(
                    diags[0].span.line_start, 2,
                    "span should be inside the expression"
                );
            }
        });
    }

    #[test]
    fn five_thousand_deep_nesting_reports_single_error() {
        on_big_stack(|| {
            for (open, close, atom) in nesting_shapes() {
                let src = nested_expr_src(open, close, atom, 5000);
                let out = parse_source(&src);
                let diags = out.diagnostics.as_slice();
                assert_eq!(
                    diags.len(),
                    1,
                    "shape {open}...{close} at 5000 deep should report exactly one diagnostic:\n{}",
                    out.diagnostics.to_golden()
                );
                assert_eq!(diags[0].code, "E1415");
                assert_eq!(
                    out.ast.declarations.len(),
                    1,
                    "main() should still be parsed"
                );
            }
        });
    }

    #[test]
    fn deep_unary_chain_reports_single_error() {
        on_big_stack(|| {
            let src = format!("main() {{\n  v := {}1\n  v\n}}\n", "-".repeat(5000));
            let out = parse_source(&src);
            let diags = out.diagnostics.as_slice();
            assert_eq!(diags.len(), 1, "{}", out.diagnostics.to_golden());
            assert_eq!(diags[0].code, "E1415");
        });
    }

    #[test]
    fn unbalanced_deep_nesting_reports_depth_error_without_crash() {
        on_big_stack(|| {
            // 5000 opens, no closers: the guard must fire and the parser
            // must finish without a stack overflow or an error cascade.
            let src = format!("main() {{\n  v := {}1\n  v\n}}\n", "(".repeat(5000));
            let out = parse_source(&src);
            let diags = out.diagnostics.as_slice();
            assert_eq!(diags[0].code, "E1415", "{}", out.diagnostics.to_golden());
            assert!(
                diags.len() <= 2,
                "unwind must not cascade:\n{}",
                out.diagnostics.to_golden()
            );
        });
    }

    #[test]
    fn parsing_recovers_after_nesting_overflow() {
        on_big_stack(|| {
            // The over-deep expression in a() must not swallow b() or
            // mute its genuine diagnostics.
            let deep = nested_expr_src("(", ")", "1", super::MAX_EXPR_NESTING_DEPTH);
            let src = format!("{deep}b() {{\n  w := 1 +\n  w\n}}\n");
            let out = parse_source(&src);
            let codes: Vec<&str> = out
                .diagnostics
                .as_slice()
                .iter()
                .map(|d| d.code.as_str())
                .collect();
            assert_eq!(
                codes,
                vec!["E1415", "E1402"],
                "{}",
                out.diagnostics.to_golden()
            );
            assert_eq!(out.ast.declarations.len(), 2, "both functions should parse");
        });
    }
}
