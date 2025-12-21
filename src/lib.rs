use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use regex::Regex;

type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    records: HashMap<String, Value>,
    parent: Option<EnvRef>,
}

impl Environment {
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Self {
            records: HashMap::new(),
            parent: None,
        }))
    }

    pub fn extend(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            records: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.records
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.borrow().get(name))
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.records.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), &'static str> {
        // Try to update in current scope
        if self.records.contains_key(name) {
            self.records.insert(name.to_string(), value);
            return Ok(());
        }
        
        // Try to update in parent scope
        if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)?;
            return Ok(());
        }
        
        // Variable not found in any scope
        Err("undefined variable")
    }
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Null,
    Int(i64),
    Float(f64),
    Str(String),
    Binary(Operation, Box<Expr>, Box<Expr>),
    Set(String, Box<Expr>),
    Assign(String, Box<Expr>),
    Var(String),
    Block(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Str(String),
}

pub struct Eva {
    globals: EnvRef,
}

impl Eva {
    pub fn new() -> Self {
        Self {
            globals: Environment::new(),
        }
    }

    pub fn eval(&self, expr: Expr) -> Result<Value, &'static str> {
        self.eval_with_env(expr, None)
    }

    pub fn eval_in_env(
        &self,
        expr: Expr,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        self.eval_with_env(expr, Some(env))
    }

    fn is_valid_var_name(name: &str) -> bool {
        let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
        re.is_match(name)
    }

    fn eval_with_env(
        &self,
        expr: Expr,
        env: Option<EnvRef>,
    ) -> Result<Value, &'static str> {
        let env = env.unwrap_or_else(|| self.globals.clone());

        match expr {
            Expr::Null => Ok(Value::Null),
            Expr::Int(n) => Ok(Value::Int(n)),
            Expr::Float(f) => Ok(Value::Float(f)),
            Expr::Str(s) => Ok(Value::Str(s)),

            Expr::Var(name) => {
                if !Self::is_valid_var_name(&name) {
                    return Err("invalid variable name");
                }
                env.borrow().get(&name).ok_or("undefined variable")
            }

            Expr::Set(name, expr) => {
                if !Self::is_valid_var_name(&name) {
                    return Err("invalid variable name");
                }
                let value = self.eval_with_env(*expr, Some(env.clone()))?;
                env.borrow_mut().set(name, value.clone());
                Ok(value)
            }

            Expr::Assign(name, expr) => {
                if !Self::is_valid_var_name(&name) {
                    return Err("invalid variable name");
                }
                let value = self.eval_with_env(*expr, Some(env.clone()))?;
                env.borrow_mut().assign(&name, value.clone())?;
                Ok(value)
            }

            Expr::Binary(op, lhs, rhs) => {
                let left = self.eval_with_env(*lhs, Some(env.clone()))?;
                let right = self.eval_with_env(*rhs, Some(env))?;
                Self::eval_binary(op, left, right)
            }

            Expr::Block(exprs) => {
                let block_scope = Environment::extend(env);
                let mut result = Value::Null; // default value for empty block
                for expr in exprs {
                    result = self.eval_with_env(expr, Some(block_scope.clone()))?;
                }
                Ok(result)
            }
        }
    }

    fn eval_binary(
        op: Operation,
        left: Value,
        right: Value,
    ) -> Result<Value, &'static str> {
        use Operation::*;
        use Value::*;

        match (op, left, right) {
            // Int
            (Add, Int(a), Int(b)) => Ok(Int(a + b)),
            (Sub, Int(a), Int(b)) => Ok(Int(a - b)),
            (Mul, Int(a), Int(b)) => Ok(Int(a * b)),
            (Div, Int(a), Int(b)) => {
                if b == 0 {
                    Err("division by zero")
                } else {
                    Ok(Int(a / b))
                }
            }

            // Float
            (Add, Float(a), Float(b)) => Ok(Float(a + b)),
            (Sub, Float(a), Float(b)) => Ok(Float(a - b)),
            (Mul, Float(a), Float(b)) => Ok(Float(a * b)),
            (Div, Float(a), Float(b)) => {
                if b == 0.0 {
                    Err("division by zero")
                } else {
                    Ok(Float(a / b))
                }
            }

            _ => Err("type error"),
        }
    }
}
