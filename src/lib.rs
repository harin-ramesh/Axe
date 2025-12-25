use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use regex::Regex;

mod parser;
pub use parser::Parser;

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

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Gt,      // >
    Lt,      // <
    Gte,     // >=
    Lte,     // <=
    Eq,      // ==
    Neq,     // !=
}

#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Binary(Operation, Box<Condition>, Box<Condition>),
    Var(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Binary(Operation, Box<Expr>, Box<Expr>),
    Set(String, Box<Expr>),
    Assign(String, Box<Expr>),
    Var(String),
    Block(Vec<Expr>),
    If(Condition, Vec<Expr>, Vec<Expr>),
    While(Condition, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

pub struct Axe {
    globals: EnvRef,
}

impl Axe {
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
            Expr::Bool(b) => Ok(Value::Bool(b)),
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

            Expr::If(condition, then_branch, else_branch) => {
                let cond_value = self.eval_condition(condition, Some(env.clone()))?;
                
                // Determine truthiness: Null, Bool(false), Int(0), and Float(0.0) are falsy
                let is_truthy = match cond_value {
                    Value::Null => false,
                    Value::Bool(b) => b,
                    Value::Int(0) => false,
                    Value::Float(f) if f == 0.0 => false,
                    _ => true,
                };
                
                // Evaluate the appropriate branch
                let branch_exprs = if is_truthy { then_branch } else { else_branch };
                let branch_scope = Environment::extend(env);
                let mut result = Value::Null;
                for expr in branch_exprs {
                    result = self.eval_with_env(expr, Some(branch_scope.clone()))?;
                }
                Ok(result)
            }

            Expr::While(condition, body) => {
                let loop_scope = Environment::extend(env);
                let mut result = Value::Null;

                loop {
                    let cond_value = self.eval_condition(condition.clone(), Some(loop_scope.clone()))?;
                    
                    // Determine truthiness: Null, Bool(false), Int(0), and Float(0.0) are falsy
                    let is_truthy = match cond_value {
                        Value::Null => false,
                        Value::Bool(b) => b,
                        Value::Int(0) => false,
                        Value::Float(f) if f == 0.0 => false,
                        _ => true,
                    };

                    if !is_truthy {
                        break;
                    }

                    // Execute loop body
                    for expr in &body {
                        result = self.eval_with_env(expr.clone(), Some(loop_scope.clone()))?;
                    }
                }

                Ok(result)
            }
        }
    }

    fn eval_condition(
        &self,
        condition: Condition,
        env: Option<EnvRef>,
    ) -> Result<Value, &'static str> {
        let env = env.unwrap_or_else(|| self.globals.clone());

        match condition {
            Condition::Null => Ok(Value::Null),
            Condition::Bool(b) => Ok(Value::Bool(b)),
            Condition::Int(n) => Ok(Value::Int(n)),
            Condition::Float(f) => Ok(Value::Float(f)),
            Condition::Str(s) => Ok(Value::Str(s)),

            Condition::Var(name) => {
                if !Self::is_valid_var_name(&name) {
                    return Err("invalid variable name");
                }
                env.borrow().get(&name).ok_or("undefined variable")
            }

            Condition::Binary(op, lhs, rhs) => {
                let left = self.eval_condition(*lhs, Some(env.clone()))?;
                let right = self.eval_condition(*rhs, Some(env))?;
                Self::eval_binary(op, left, right)
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

            // Comparison operations for Int
            (Gt, Int(a), Int(b)) => Ok(Bool(a > b)),
            (Lt, Int(a), Int(b)) => Ok(Bool(a < b)),
            (Gte, Int(a), Int(b)) => Ok(Bool(a >= b)),
            (Lte, Int(a), Int(b)) => Ok(Bool(a <= b)),
            (Eq, Int(a), Int(b)) => Ok(Bool(a == b)),
            (Neq, Int(a), Int(b)) => Ok(Bool(a != b)),

            // Comparison operations for Float
            (Gt, Float(a), Float(b)) => Ok(Bool(a > b)),
            (Lt, Float(a), Float(b)) => Ok(Bool(a < b)),
            (Gte, Float(a), Float(b)) => Ok(Bool(a >= b)),
            (Lte, Float(a), Float(b)) => Ok(Bool(a <= b)),
            (Eq, Float(a), Float(b)) => Ok(Bool(a == b)),
            (Neq, Float(a), Float(b)) => Ok(Bool(a != b)),

            // Equality operations for String
            (Eq, Str(ref a), Str(ref b)) => Ok(Bool(a == b)),
            (Neq, Str(ref a), Str(ref b)) => Ok(Bool(a != b)),

            // Equality operations for Bool
            (Eq, Bool(a), Bool(b)) => Ok(Bool(a == b)),
            (Neq, Bool(a), Bool(b)) => Ok(Bool(a != b)),

            // Equality operations for Null
            (Eq, Null, Null) => Ok(Bool(true)),
            (Neq, Null, Null) => Ok(Bool(false)),

            // Cross-type equality checks (always false for Eq, true for Neq)
            (Eq, _, _) => Ok(Bool(false)),
            (Neq, _, _) => Ok(Bool(true)),

            _ => Err("type error"),
        }
    }
}
