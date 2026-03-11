// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use vibe_ast::{
    BinaryOp, Contract, Declaration, EnumDecl, ExampleCase, Expr, FileAst, FunctionDecl, MatchArm,
    Param, SelectCase, SelectPattern, Stmt, TypeDecl, TypeField, TypeRef, UnaryOp,
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

struct Parser {
    tokens: Vec<Token>,
    idx: usize,
    diagnostics: Diagnostics,
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
            let variant = self.expect_ident("E1142", "expected variant name");
            variants.push(variant);
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

    fn parse_function(&mut self) -> FunctionDecl {
        let start = self.peek().span;
        let name = self.expect_ident("E1102", "expected function name");
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
            params,
            return_type,
            contracts,
            body,
            tail_expr,
            span: Span::new(start.line_start, start.col_start, end.line_end, end.col_end),
        }
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        self.consume_newlines();
        while !self.at(&TokenKind::RParen) && !self.is_eof() {
            let name = self.expect_ident("E1107", "expected parameter name");
            let ty = if self.match_kind(&TokenKind::Colon) {
                Some(self.parse_type_ref_until(&[TokenKind::Comma, TokenKind::RParen]))
            } else {
                None
            };
            params.push(Param { name, ty });
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
        while !self.is_eof() {
            if angle_depth == 0 && stops.iter().any(|s| self.at(s)) {
                break;
            }
            if angle_depth == 0 && self.at(&TokenKind::Newline) {
                break;
            }
            let tok = self.bump();
            match tok.kind {
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

        if let Some(stmt) = self.try_parse_binding(start) {
            return Some(stmt);
        }
        if let Some(stmt) = self.try_parse_assignment(start) {
            return Some(stmt);
        }

        let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
        Some(Stmt::ExprStmt { expr, span: start })
    }

    fn try_parse_binding(&mut self, span: Span) -> Option<Stmt> {
        if !(self.at_ident() && self.peek_n_kind(1) == Some(&TokenKind::Bind)) {
            return None;
        }
        let name = self.bump().lexeme;
        self.bump(); // :=
        let expr = self.parse_expr_until(&[StopToken::Newline, StopToken::RBrace]);
        Some(Stmt::Binding { name, expr, span })
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

    fn parse_binary_expr(&mut self, min_prec: u8, stop: &[StopToken]) -> Option<Expr> {
        let mut left = self.parse_unary_expr(stop)?;
        loop {
            if self.is_stop(stop) {
                break;
            }
            let (op, prec) = match self.peek().kind {
                TokenKind::Star => (BinaryOp::Mul, 40),
                TokenKind::Slash => (BinaryOp::Div, 40),
                TokenKind::Plus => (BinaryOp::Add, 30),
                TokenKind::Minus => (BinaryOp::Sub, 30),
                TokenKind::EqEq => (BinaryOp::Eq, 20),
                TokenKind::NotEq => (BinaryOp::Ne, 20),
                TokenKind::Lt => (BinaryOp::Lt, 25),
                TokenKind::Le => (BinaryOp::Le, 25),
                TokenKind::Gt => (BinaryOp::Gt, 25),
                TokenKind::Ge => (BinaryOp::Ge, 25),
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
                let inner = self.parse_unary_expr(stop).unwrap_or_else(|| {
                    self.error_expr("E1403", "expected expression after unary `-`")
                });
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
                let inner = self.parse_unary_expr(stop).unwrap_or_else(|| {
                    self.error_expr("E1404", "expected expression after unary `!`")
                });
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
                let inner = self.parse_unary_expr(stop).unwrap_or_else(|| {
                    self.error_expr("E1404A", "expected expression after `async`")
                });
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
                let inner = self.parse_unary_expr(stop).unwrap_or_else(|| {
                    self.error_expr("E1404B", "expected expression after `await`")
                });
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
                Expr::Int {
                    value: t.lexeme.parse().unwrap_or_default(),
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
                Expr::String {
                    value: t.lexeme,
                    span: t.span,
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
                        self.diagnostics.push(Diagnostic::new(
                            "E1411",
                            Severity::Error,
                            "expected `:` after map key",
                            self.peek().span,
                        ));
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

    fn error_expr(&mut self, code: &str, message: &str) -> Expr {
        let span = self.peek().span;
        self.diagnostics
            .push(Diagnostic::new(code, Severity::Error, message, span));
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
            self.diagnostics
                .push(Diagnostic::new(code, Severity::Error, message, span));
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
}
