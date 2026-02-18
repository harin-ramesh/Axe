//! Runtime values for the tree-walking interpreter.

use crate::ast::{Literal, Stmt};
use crate::context::Context;
use crate::interner::Symbol;

use super::environment::EnvRef;

/// Type alias for native function signature.
/// Native functions receive the context (for string interning) and arguments.
pub type NativeFn = fn(&Context, &[Value]) -> Result<Value, super::interpreter::EvalSignal>;

#[derive(Debug)]
pub enum Value {
    Literal(Literal),
    List(Vec<Value>),
    Function(Vec<Symbol>, Box<Stmt>, EnvRef),
    NativeFunction(Symbol, NativeFn),
    Object(EnvRef),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Literal(lit) => Value::Literal(*lit),
            Value::List(items) => Value::List(items.clone()),
            Value::Function(params, body, env) => {
                Value::Function(params.clone(), body.clone(), env.clone())
            }
            Value::NativeFunction(name, func) => Value::NativeFunction(*name, *func),
            Value::Object(env) => Value::Object(env.clone()),
        }
    }
}

/// Display implementation that requires a context to resolve symbols.
impl Value {
    pub fn display(&self, ctx: &crate::context::Context) -> String {
        match self {
            Value::Literal(lit) => match lit {
                Literal::Null => "null".to_string(),
                Literal::Bool(b) => format!("{}", b),
                Literal::Int(n) => format!("{}", n),
                Literal::Float(fl) => format!("{}", fl),
                Literal::Str(s) => format!("\"{}\"", ctx.resolve(*s)),
            },
            Value::List(items) => {
                let item_strs: Vec<String> = items.iter().map(|item| item.display(ctx)).collect();
                format!("[{}]", item_strs.join(", "))
            }
            Value::Function(params, _, _) => {
                let param_strs: Vec<String> = params.iter().map(|p| ctx.resolve(*p)).collect();
                format!("<function({})>", param_strs.join(", "))
            }
            Value::NativeFunction(name, _) => {
                format!("<native-function {}>", ctx.resolve(*name))
            }
            Value::Object(_) => "<object>".to_string(),
        }
    }

    /// Display without context - for debugging only, shows symbol IDs instead of names.
    pub fn display_debug(&self) -> String {
        match self {
            Value::Literal(lit) => match lit {
                Literal::Null => "null".to_string(),
                Literal::Bool(b) => format!("{}", b),
                Literal::Int(n) => format!("{}", n),
                Literal::Float(fl) => format!("{}", fl),
                Literal::Str(s) => format!("\"<sym:{}>\"", s.id()),
            },
            Value::List(items) => {
                let item_strs: Vec<String> =
                    items.iter().map(|item| item.display_debug()).collect();
                format!("[{}]", item_strs.join(", "))
            }
            Value::Function(_, _, _) => "<function>".to_string(),
            Value::NativeFunction(name, _) => format!("<native-function sym:{}>", name.id()),
            Value::Object(_) => "<object>".to_string(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Fallback display that shows symbol IDs - use display() with interner for real output
        write!(f, "{}", self.display_debug())
    }
}
