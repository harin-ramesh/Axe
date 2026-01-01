use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use regex::Regex;

mod parser;
mod transformer;

pub use parser::Parser;
pub use transformer::Transformer;

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

    pub fn update(&mut self, name: String, value: Value) -> Result<(), &'static str> {
        // Search for the variable in current scope
        if self.records.contains_key(&name) {
            self.records.insert(name, value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            // Search in parent scope
            parent.borrow_mut().update(name, value)
        } else {
            // Variable not found in any scope
            Err("undefined variable")
        }
    }

    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        self.records.contains_key(name)
    }

    pub fn exists_in_any_scope(&self, name: &str) -> bool {
        self.records.contains_key(name)
            || self.parent.as_ref().map_or(false, |p| p.borrow().exists_in_any_scope(name))
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
    List(Vec<Expr>),
    Binary(Operation, Box<Expr>, Box<Expr>),
    Set(String, Box<Expr>),
    Var(String),
    Block(Vec<Expr>),
    If(Condition, Vec<Expr>, Vec<Expr>),
    While(Condition, Vec<Expr>),
    Lambda(Vec<String>, Vec<Expr>),
    Function(String, Vec<String>, Vec<Expr>),
    FunctionCall(String, Vec<Expr>),
    Inc(String),  // i++ -> (let i (+ i 1))
    Dec(String),  // i-- -> (let i (- i 1))
    For(Box<Expr>, Condition, Box<Expr>, Vec<Expr>),  // (for init condition update body...) -> while loop
}

pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Function(Vec<String>, Vec<Expr>, EnvRef),
    NativeFunction(String, fn(&[Value]) -> Result<Value, &'static str>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Null => Value::Null,
            Value::Bool(b) => Value::Bool(*b),
            Value::Int(n) => Value::Int(*n),
            Value::Float(f) => Value::Float(*f),
            Value::Str(s) => Value::Str(s.clone()),
            Value::List(items) => Value::List(items.clone()),
            Value::Function(p, b, e) => Value::Function(p.clone(), b.clone(), e.clone()),
            Value::NativeFunction(name, func) => Value::NativeFunction(name.clone(), *func),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "Null"),
            Value::Bool(b) => write!(f, "Bool({:?})", b),
            Value::Int(n) => write!(f, "Int({:?})", n),
            Value::Float(fl) => write!(f, "Float({:?})", fl),
            Value::Str(s) => write!(f, "Str({:?})", s),
            Value::List(items) => write!(f, "List({:?})", items),
            Value::Function(params, body, _) => write!(f, "Function({:?}, {:?}, <env>)", params, body),
            Value::NativeFunction(name, _) => write!(f, "NativeFunction({})", name),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Function(..), Value::Function(..)) => false,
            (Value::NativeFunction(name_a, _), Value::NativeFunction(name_b, _)) => {
                name_a == name_b
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
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Function(params, _, _) => {
                write!(f, "<function({})", params.join(", "))?;
                write!(f, ">")
            }
            Value::NativeFunction(name, _) => write!(f, "<native-fn:{}>", name),
        }
    }
}

// Built-in native functions
fn native_print(args: &[Value]) -> Result<Value, &'static str> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        // Print strings without quotes, everything else with Display
        match arg {
            Value::Str(s) => print!("{}", s),
            _ => print!("{}", arg),
        }
    }
    println!();
    Ok(Value::Null)
}

fn native_len(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 {
        return Err("len expects exactly 1 argument");
    }
    match &args[0] {
        Value::List(items) => Ok(Value::Int(items.len() as i64)),
        Value::Str(s) => Ok(Value::Int(s.len() as i64)),
        _ => Err("len expects a list or string"),
    }
}

fn native_push(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 {
        return Err("push expects exactly 2 arguments");
    }
    match &args[0] {
        Value::List(items) => {
            let mut new_list = items.clone();
            new_list.push(args[1].clone());
            Ok(Value::List(new_list))
        }
        _ => Err("push expects a list as first argument"),
    }
}

fn native_get(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 {
        return Err("get expects exactly 2 arguments");
    }
    match (&args[0], &args[1]) {
        (Value::List(items), Value::Int(idx)) => {
            let index = if *idx < 0 {
                (items.len() as i64 + idx) as usize
            } else {
                *idx as usize
            };
            items.get(index).cloned().ok_or("index out of bounds")
        }
        _ => Err("get expects a list and an integer index"),
    }
}

fn native_type(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 {
        return Err("type expects exactly 1 argument");
    }
    let type_name = match &args[0] {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Str(_) => "string",
        Value::List(_) => "list",
        Value::Function(..) => "function",
        Value::NativeFunction(..) => "native-function",
    };
    Ok(Value::Str(type_name.to_string()))
}

fn native_concat(args: &[Value]) -> Result<Value, &'static str> {
    if args.is_empty() {
        return Err("concat expects at least 1 argument");
    }
    
    // Check if first argument is a string or list
    match &args[0] {
        Value::Str(_) => {
            // Concatenate strings
            let mut result = String::new();
            for arg in args {
                match arg {
                    Value::Str(s) => result.push_str(s),
                    _ => return Err("concat on strings requires all arguments to be strings"),
                }
            }
            Ok(Value::Str(result))
        }
        Value::List(_) => {
            // Concatenate lists
            let mut result = Vec::new();
            for arg in args {
                match arg {
                    Value::List(items) => result.extend(items.clone()),
                    _ => return Err("concat on lists requires all arguments to be lists"),
                }
            }
            Ok(Value::List(result))
        }
        _ => Err("concat expects strings or lists"),
    }
}

fn native_range(args: &[Value]) -> Result<Value, &'static str> {
    match args.len() {
        1 => {
            // range(n) -> [0, 1, ..., n-1]
            match &args[0] {
                Value::Int(n) => {
                    if *n < 0 {
                        return Err("range expects non-negative integer");
                    }
                    let values: Vec<Value> = (0..*n).map(Value::Int).collect();
                    Ok(Value::List(values))
                }
                _ => Err("range expects integer argument"),
            }
        }
        2 => {
            // range(start, end) -> [start, start+1, ..., end-1]
            match (&args[0], &args[1]) {
                (Value::Int(start), Value::Int(end)) => {
                    let values: Vec<Value> = (*start..*end).map(Value::Int).collect();
                    Ok(Value::List(values))
                }
                _ => Err("range expects integer arguments"),
            }
        }
        3 => {
            // range(start, end, step) -> [start, start+step, ..., end-step]
            match (&args[0], &args[1], &args[2]) {
                (Value::Int(start), Value::Int(end), Value::Int(step)) => {
                    if *step == 0 {
                        return Err("range step cannot be zero");
                    }
                    let mut values = Vec::new();
                    let mut current = *start;
                    if *step > 0 {
                        while current < *end {
                            values.push(Value::Int(current));
                            current += step;
                        }
                    } else {
                        while current > *end {
                            values.push(Value::Int(current));
                            current += step;
                        }
                    }
                    Ok(Value::List(values))
                }
                _ => Err("range expects integer arguments"),
            }
        }
        _ => Err("range expects 1, 2, or 3 arguments"),
    }
}

pub struct Axe {
    globals: EnvRef,
    transformer: Transformer,
}

impl Axe {
    pub fn new() -> Self {
        let globals = Environment::new();
        
        // Add built-in functions
        globals.borrow_mut().set(
            "print".to_string(),
            Value::NativeFunction("print".to_string(), native_print),
        );
        globals.borrow_mut().set(
            "len".to_string(),
            Value::NativeFunction("len".to_string(), native_len),
        );
        globals.borrow_mut().set(
            "push".to_string(),
            Value::NativeFunction("push".to_string(), native_push),
        );
        globals.borrow_mut().set(
            "get".to_string(),
            Value::NativeFunction("get".to_string(), native_get),
        );
        globals.borrow_mut().set(
            "type".to_string(),
            Value::NativeFunction("type".to_string(), native_type),
        );
        globals.borrow_mut().set(
            "concat".to_string(),
            Value::NativeFunction("concat".to_string(), native_concat),
        );
        globals.borrow_mut().set(
            "range".to_string(),
            Value::NativeFunction("range".to_string(), native_range),
        );
        
        Self { 
            globals,
            transformer: Transformer,
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
            
            Expr::List(elements) => {
                // Evaluate each element and create a list
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_with_env(elem, Some(env.clone()))?);
                }
                Ok(Value::List(values))
            }

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
                
                // Try to update existing variable first (searches parent scopes)
                // If not found, create new variable in current scope
                if env.borrow_mut().update(name.clone(), value.clone()).is_err() {
                    env.borrow_mut().set(name, value.clone());
                }
                
                Ok(value)
            }

            Expr::Binary(op, lhs, rhs) => {
                let left = self.eval_with_env(*lhs, Some(env.clone()))?;
                let right = self.eval_with_env(*rhs, Some(env))?;
                Self::eval_binary(op, left, right)
            }

            Expr::Block(exprs) => {
                // Don't create a new scope - execute in the current environment
                // This allows variables to be updated within blocks
                let mut result = Value::Null; // default value for empty block
                for expr in exprs {
                    result = self.eval_with_env(expr, Some(env.clone()))?;
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
                
                // Evaluate the appropriate branch in current environment
                let branch_exprs = if is_truthy { then_branch } else { else_branch };
                let mut result = Value::Null;
                for expr in branch_exprs {
                    result = self.eval_with_env(expr, Some(env.clone()))?;
                }
                Ok(result)
            }

            Expr::While(condition, body) => {
                // Don't create a new scope - execute in the current environment
                // This allows loop variables to be updated
                let mut result = Value::Null;

                loop {
                    let cond_value = self.eval_condition(condition.clone(), Some(env.clone()))?;
                    
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

                    // Execute loop body in current environment
                    for expr in &body {
                        result = self.eval_with_env(expr.clone(), Some(env.clone()))?;
                    }
                }

                Ok(result)
            }

            Expr::Lambda(params, body) => {
                // Validate parameter names
                for param in &params {
                    if !Self::is_valid_var_name(param) {
                        return Err("invalid parameter name");
                    }
                }
                
                // Create a closure capturing the current environment
                let func_value = Value::Function(params.clone(), body.clone(), env.clone());
                
                Ok(func_value)
            }

            Expr::Function(name, params, body) => {
                // Transform syntactic sugar: (fn name params body) -> (let name (lambda params body))
                // Then evaluate the transformed expression
                let transformed = self.transformer.transform(Expr::Function(name, params, body));
                self.eval_with_env(transformed, Some(env))
            }

            Expr::Inc(var) => {
                // Transform syntactic sugar: i++ -> (let i (+ i 1))
                // Then evaluate the transformed expression
                let transformed = self.transformer.transform(Expr::Inc(var));
                self.eval_with_env(transformed, Some(env))
            }

            Expr::Dec(var) => {
                // Transform syntactic sugar: i-- -> (let i (- i 1))
                // Then evaluate the transformed expression
                let transformed = self.transformer.transform(Expr::Dec(var));
                self.eval_with_env(transformed, Some(env))
            }

            Expr::For(init, condition, update, body) => {
                // Transform syntactic sugar: for loop -> while loop
                // Then evaluate the transformed expression
                let transformed = self.transformer.transform(Expr::For(init, condition, update, body));
                self.eval_with_env(transformed, Some(env))
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
                    Value::NativeFunction(_, native_fn) => {
                        // Evaluate arguments in the caller's environment
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.eval_with_env(arg, Some(env.clone()))?);
                        }

                        // Call the native function
                        native_fn(&arg_values)
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
                    Value::NativeFunction(_, native_fn) => {
                        // Evaluate arguments in the caller's environment
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.eval_condition(arg, Some(env.clone()))?);
                        }

                        // Call the native function
                        native_fn(&arg_values)
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
