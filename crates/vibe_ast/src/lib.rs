// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use vibe_diagnostics::Span;

#[derive(Debug, Clone, Default)]
pub struct FileAst {
    pub module: Option<String>,
    pub imports: Vec<String>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Function(FunctionDecl),
    Type(TypeDecl),
    Enum(EnumDecl),
}

#[derive(Debug, Clone, Default)]
pub struct TypeDecl {
    pub is_public: bool,
    pub name: String,
    pub fields: Vec<TypeField>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct TypeField {
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, Default)]
pub struct EnumDecl {
    pub is_public: bool,
    pub name: String,
    pub variants: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct FunctionDecl {
    pub is_public: bool,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub contracts: Vec<Contract>,
    pub body: Vec<Stmt>,
    pub tail_expr: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct Param {
    pub name: String,
    pub ty: Option<TypeRef>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeRef {
    pub raw: String,
}

#[derive(Debug, Clone)]
pub enum Contract {
    Intent { text: String, span: Span },
    Examples { cases: Vec<ExampleCase>, span: Span },
    Require { expr: Expr, span: Span },
    Ensure { expr: Expr, span: Span },
    Effect { name: String, span: Span },
    Native { symbol: String, span: Span },
}

#[derive(Debug, Clone)]
pub struct ExampleCase {
    pub call: Expr,
    pub expected: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Binding {
        name: String,
        expr: Expr,
        span: Span,
    },
    Assignment {
        target: Expr,
        expr: Expr,
        span: Span,
    },
    Return {
        expr: Expr,
        span: Span,
    },
    ExprStmt {
        expr: Expr,
        span: Span,
    },
    For {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Repeat {
        count: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Select {
        cases: Vec<SelectCase>,
        span: Span,
    },
    Go {
        expr: Expr,
        span: Span,
    },
    Thread {
        expr: Expr,
        span: Span,
    },
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
        default_action: Option<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Expr,
    pub action: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum SelectPattern {
    Receive { binding: String, expr: Expr },
    After { duration_literal: String },
    Closed { ident: String },
    Default,
}

#[derive(Debug, Clone)]
pub struct SelectCase {
    pub pattern: SelectPattern,
    pub action: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Ident {
        name: String,
        span: Span,
    },
    Int {
        value: i64,
        span: Span,
    },
    Float {
        value: f64,
        span: Span,
    },
    Bool {
        value: bool,
        span: Span,
    },
    String {
        value: String,
        span: Span,
    },
    List {
        items: Vec<Expr>,
        span: Span,
    },
    Map {
        entries: Vec<(Expr, Expr)>,
        span: Span,
    },
    Member {
        object: Box<Expr>,
        field: String,
        span: Span,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Async {
        expr: Box<Expr>,
        span: Span,
    },
    Await {
        expr: Box<Expr>,
        span: Span,
    },
    Question {
        expr: Box<Expr>,
        span: Span,
    },
    DotResult {
        span: Span,
    },
    Old {
        expr: Box<Expr>,
        span: Span,
    },
    Constructor {
        type_name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident { span, .. }
            | Expr::Int { span, .. }
            | Expr::Float { span, .. }
            | Expr::Bool { span, .. }
            | Expr::String { span, .. }
            | Expr::List { span, .. }
            | Expr::Map { span, .. }
            | Expr::Member { span, .. }
            | Expr::Index { span, .. }
            | Expr::Slice { span, .. }
            | Expr::Call { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Async { span, .. }
            | Expr::Await { span, .. }
            | Expr::Question { span, .. }
            | Expr::DotResult { span }
            | Expr::Old { span, .. }
            | Expr::Constructor { span, .. }
            | Expr::EnumVariant { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}
