//! Abstract Syntax Tree (AST) definitions for the Axe language.
//!
//! This module contains pure data structures representing the parsed
//! program structure. It is independent of any execution strategy,
//! making it suitable for use with both a tree-walking interpreter
//! and a future bytecode VM.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::interner::Symbol;

/// Global counter for generating unique expression IDs.
static EXPR_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique identifier for expression nodes, used for variable resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub u64);

impl ExprId {
    pub fn new() -> Self {
        Self(EXPR_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for ExprId {
    fn default() -> Self {
        Self::new()
    }
}

/// A complete Axe program consisting of a list of statements.
#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/// Literal values that can appear directly in source code.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Symbol),
}

/// Binary operations supported by the language.
#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Gt,  // >
    Lt,  // <
    Gte, // >=
    Lte, // <=
    Eq,  // ==
    Neq, // !=
    And,
    Or,
    BitwiseAnd,
    BitwiseOr,
    Mod,
}

/// Unary operations.
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Neg, // -x (numeric negation)
    Not, // !x (logical not)
    Inv, // ~x (bitwise invert)
}

/// Expression node wrapper with unique ID for variable resolution.
#[derive(Debug, Clone)]
pub struct Expr {
    pub id: ExprId,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self {
            id: ExprId::new(),
            kind,
        }
    }

    // Convenience constructors for backward compatibility with old Expr::Variant(...) syntax
    // These allow existing code using `Expr::Var(...)` to continue working.
    #[allow(non_snake_case)]
    #[inline]
    pub fn Literal(lit: Literal) -> Self {
        Self::new(ExprKind::Literal(lit))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn List(elements: Vec<Expr>) -> Self {
        Self::new(ExprKind::List(elements))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn Var(name: Symbol) -> Self {
        Self::new(ExprKind::Var(name))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn Binary(op: Operation, lhs: Box<Expr>, rhs: Box<Expr>) -> Self {
        Self::new(ExprKind::Binary(op, lhs, rhs))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn Unary(op: UnaryOp, operand: Box<Expr>) -> Self {
        Self::new(ExprKind::Unary(op, operand))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn Call(name: Symbol, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::Call(name, args))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn Lambda(params: Vec<Symbol>, body: Box<Stmt>) -> Self {
        Self::new(ExprKind::Lambda(params, body))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn New(class: Symbol, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::New(class, args))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn Property(obj: Box<Expr>, name: Symbol) -> Self {
        Self::new(ExprKind::Property(obj, name))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn MethodCall(obj: Box<Expr>, method: Symbol, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::MethodCall(obj, method, args))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn StaticProperty(obj: Box<Expr>, name: Symbol) -> Self {
        Self::new(ExprKind::StaticProperty(obj, name))
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn StaticMethodCall(obj: Box<Expr>, method: Symbol, args: Vec<Expr>) -> Self {
        Self::new(ExprKind::StaticMethodCall(obj, method, args))
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        // Only compare the kind, not the ID (IDs are unique per instance)
        self.kind == other.kind
    }
}

/// Expression node variants in the AST.
#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    /// A literal value (number, string, bool, null)
    Literal(Literal),
    /// A list literal: [expr, expr, ...]
    List(Vec<Expr>),
    /// A variable reference
    Var(Symbol),
    /// A binary operation: lhs op rhs
    Binary(Operation, Box<Expr>, Box<Expr>),
    /// A unary operation: op expr
    Unary(UnaryOp, Box<Expr>),
    /// A function call: name(args...)
    Call(Symbol, Vec<Expr>),
    /// A lambda expression: |params| body
    Lambda(Vec<Symbol>, Box<Stmt>),
    /// Object instantiation: new ClassName(args...)
    New(Symbol, Vec<Expr>),
    /// Property access: obj.property
    Property(Box<Expr>, Symbol),
    /// Method call: obj.method(args...)
    MethodCall(Box<Expr>, Symbol, Vec<Expr>),
    /// Static Property access: Class::property
    StaticProperty(Box<Expr>, Symbol),
    /// Static Method call: Class.method(args...)
    StaticMethodCall(Box<Expr>, Symbol, Vec<Expr>),
}

/// Statement nodes in the AST.
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// An expression statement
    Expr(Expr),
    /// A block of statements: { stmt; stmt; ... }
    Block(Vec<Stmt>),
    /// Variable declaration: let name = expr, name2 = expr2;
    Let(Vec<(Symbol, Option<Expr>, Option<Expr>)>),
    /// Variable assignment: name = expr;
    Assign(Symbol, Expr),
    /// Conditional: if (cond) { then } else { else }
    If(Expr, Box<Stmt>, Box<Stmt>),
    /// While loop: while (cond) { body }
    While(Expr, Box<Stmt>),
    /// For loop: for var in iterable { body }
    For(Symbol, Expr, Box<Stmt>),
    /// Function declaration: fn name(params) { body }
    Function(Symbol, Vec<Symbol>, Box<Stmt>),
    /// Class declaration: class Name [: Parent] { body }
    Class(Symbol, Option<Symbol>, Vec<Stmt>),
    /// A return statement: return expr
    Return(Box<Expr>),
    /// A break statement: break;
    Break,
    /// A continue statement: continue;
    Continue,
    /// An import statement: import "module" [as alias1, alias2, ...];
    Import(Symbol, Vec<Symbol>),
}
