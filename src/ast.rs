//! Abstract Syntax Tree (AST) definitions for the Axe language.
//!
//! This module contains pure data structures representing the parsed
//! program structure. It is independent of any execution strategy,
//! making it suitable for use with both a tree-walking interpreter
//! and a future bytecode VM.

/// A complete Axe program consisting of a list of statements.
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/// Literal values that can appear directly in source code.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
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

/// Expression nodes in the AST.
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    /// A literal value (number, string, bool, null)
    Literal(Literal),
    /// A list literal: [expr, expr, ...]
    List(Vec<Expr>),
    /// A variable reference
    Var(String),
    /// A binary operation: lhs op rhs
    Binary(Operation, Box<Expr>, Box<Expr>),
    /// A unary operation: op expr
    Unary(UnaryOp, Box<Expr>),
    /// A function call: name(args...)
    Call(String, Vec<Expr>),
    /// A lambda expression: |params| body
    Lambda(Vec<String>, Box<Stmt>),
    /// Property access: obj.property
    Property(Box<Expr>, String),
    /// Method call: obj.method(args...)
    MethodCall(Box<Expr>, String, Vec<Expr>),
    /// Object instantiation: new ClassName(args...)
    New(String, Vec<Expr>),
}

/// Statement nodes in the AST.
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// An expression statement
    Expr(Expr),
    /// A block of statements: { stmt; stmt; ... }
    Block(Vec<Stmt>),
    /// Variable declaration: let name = expr, name2 = expr2;
    Let(Vec<(String, Option<Expr>)>),
    /// Variable assignment: name = expr;
    Assign(String, Expr),
    /// Conditional: if (cond) { then } else { else }
    If(Expr, Box<Stmt>, Box<Stmt>),
    /// While loop: while (cond) { body }
    While(Expr, Box<Stmt>),
    /// For loop: for var in iterable { body }
    For(String, Expr, Box<Stmt>),
    /// Function declaration: fn name(params) { body }
    Function(String, Vec<String>, Box<Stmt>),
    /// Class declaration: class Name [: Parent] { body }
    Class(String, Option<String>, Vec<Stmt>),
}
