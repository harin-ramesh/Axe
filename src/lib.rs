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
    FunctionCall(String, Vec<Condition>),
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
    Function(Vec<String>, Vec<Expr>), // (fn (params...) body...)
    FunctionCall(String, Vec<Expr>),  // (funcname args...)
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Function(Vec<String>, Vec<Expr>, EnvRef), // parameters, body, closure environment
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            // Functions are compared by reference equality (pointer comparison)
            (Value::Function(_, _, env_a), Value::Function(_, _, env_b)) => {
                Rc::ptr_eq(env_a, env_b)
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::Function(params, _, _) => {
                write!(f, "<function({})", params.join(", "))?;
                write!(f, ">")
            }
        }
    }
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

            Expr::Function(params, body) => {
                // Validate parameter names
                for param in &params {
                    if !Self::is_valid_var_name(param) {
                        return Err("invalid parameter name");
                    }
                }
                // Create a closure capturing the current environment
                Ok(Value::Function(params.clone(), body.clone(), env))
            }

            Expr::FunctionCall(name, args) => {
                // Get the function from the environment
                let func = env.borrow().get(&name).ok_or("undefined function")?;

                match func {
                    Value::Function(params, body, closure_env) => {
                        // Check argument count
                        if params.len() != args.len() {
                            return Err("argument count mismatch");
                        }

                        // Evaluate arguments in the caller's environment
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.eval_with_env(arg, Some(env.clone()))?);
                        }

                        // Create a new environment extending the closure environment
                        let func_env = Environment::extend(closure_env);

                        // Bind parameters to argument values
                        for (param, value) in params.iter().zip(arg_values.iter()) {
                            func_env.borrow_mut().set(param.clone(), value.clone());
                        }

                        // Execute function body
                        let mut result = Value::Null;
                        for expr in &body {
                            result = self.eval_with_env(expr.clone(), Some(func_env.clone()))?;
                        }

                        Ok(result)
                    }
                    _ => Err("not a function"),
                }
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

            Condition::FunctionCall(name, args) => {
                // Get the function from the environment
                let func = env.borrow().get(&name).ok_or("undefined function")?;

                match func {
                    Value::Function(params, body, closure_env) => {
                        // Check argument count
                        if params.len() != args.len() {
                            return Err("argument count mismatch");
                        }

                        // Evaluate arguments in the caller's environment
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.eval_condition(arg, Some(env.clone()))?);
                        }

                        // Create a new environment extending the closure environment
                        let func_env = Environment::extend(closure_env);

                        // Bind parameters to argument values
                        for (param, value) in params.iter().zip(arg_values.iter()) {
                            func_env.borrow_mut().set(param.clone(), value.clone());
                        }

                        // Execute function body
                        let mut result = Value::Null;
                        for expr in &body {
                            result = self.eval_with_env(expr.clone(), Some(func_env.clone()))?;
                        }

                        Ok(result)
                    }
                    _ => Err("not a function"),
                }
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
