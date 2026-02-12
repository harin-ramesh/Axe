//! Runtime values for the tree-walking interpreter.

use crate::ast::{Literal, Stmt};

use super::environment::EnvRef;

#[derive(Debug)]
pub enum Value {
    Literal(Literal),
    List(Vec<Value>),
    Function(Vec<String>, Box<Stmt>, EnvRef),
    NativeFunction(
        String,
        fn(&[Value]) -> Result<Value, super::interpreter::EvalSignal>,
    ),
    Object(EnvRef),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Literal(lit) => Value::Literal(lit.clone()),
            Value::List(items) => Value::List(items.clone()),
            Value::Function(params, body, env) => {
                Value::Function(params.clone(), body.clone(), env.clone())
            }
            Value::NativeFunction(name, func) => Value::NativeFunction(name.clone(), *func),
            Value::Object(env) => Value::Object(env.clone()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Literal(lit) => match lit {
                Literal::Null => write!(f, "null"),
                Literal::Bool(b) => write!(f, "{}", b),
                Literal::Int(n) => write!(f, "{}", n),
                Literal::Float(fl) => write!(f, "{}", fl),
                Literal::Str(s) => write!(f, "\"{}\"", s),
            },
            Value::List(items) => {
                let item_strs: Vec<String> = items.iter().map(|item| format!("{}", item)).collect();
                write!(f, "[{}]", item_strs.join(", "))
            }
            Value::Function(params, _, _) => {
                write!(f, "<function({})>", params.join(", "))
            }
            Value::NativeFunction(name, _) => {
                write!(f, "<native-function {}>", name)
            }
            Value::Object(_) => {
                write!(f, "<object>")
            }
        }
    }
}
