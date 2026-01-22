use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use regex::Regex;

use crate::transformer::Transformer;

pub type EnvRef = Rc<RefCell<Environment>>;

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
            || self
                .parent
                .as_ref()
                .map_or(false, |p| p.borrow().exists_in_any_scope(name))
    }
}

pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

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

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Neg,  // -x
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    List(Vec<Expr>),
    Var(String),
    Binary(Operation, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Lambda(Vec<String>, Vec<Stmt>),
    Property(Box<Expr>, String),
    New(String, Vec<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr),
    Block(Vec<Stmt>),
    Let(Vec<String, Option<Expr>>),
    Assign(String, Expr),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Stmt),
    Function(String, Vec<String>, Box<Stmt>),
    Class(String, Option<String>, Box<Stmt>),
}

#[derive(Debug)]
pub enum Value {
    Literal(Literal),
    List(Vec<Value>),
    Function(Vec<String>, Vec<Stmt>, EnvRef),
    NativeFunction(String, fn(&[Value]) -> Result<Value, &'static str>),
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

fn native_print(args: &[Value]) -> Result<Value, &'static str> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }

        match arg {
            Value::Literal(s) => {
                if let Literal::Str(s) = s {
                    print!("{}", s);
                } else {
                    print!("{}", arg);
                }
            },
            _ => print!("{}", arg),
        }
    }
    println!();
    Ok(Value::Literal(Literal::Null))
}

fn native_len(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 {
        return Err("len expects exactly 1 argument");
    }
    match &args[0] {
        Value::List(items) => Ok(Value::Literal(Literal::Int(items.len() as i64))),
        Value::Literal(s) => {
            match s {
                Literal::Str(s) => Ok(Value::Literal(Literal::Int(s.len() as i64))),
                _ => Err("len expects a list or string"),
            }
        }
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
        (Value::List(items), Value::Literal(i)) => {
            match i {
                Literal::Int(idx) => {
                    let index = if *idx < 0 {
                        (items.len() as i64 + idx) as usize
                    } else {
                        *idx as usize
                    };
                    items.get(index).cloned().ok_or("index out of bounds")
                }
                _ => Err("get expects an integer index as second argument"),
            }
        }
        _ => Err("get expects a list and an integer index"),
    }
}

fn native_type(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 {
        return Err("type expects exactly 1 argument");
    }
    let type_name = match &args[0] {
        Value::Literal(lit) => match lit {
            Literal::Null => "null",
            Literal::Bool(_) => "bool",
            Literal::Int(_) => "int",
            Literal::Float(_) => "float",
            Literal::Str(_) => "string",
        },
        Value::List(_) => "list",
        Value::Function(..) => "function",
        Value::NativeFunction(..) => "native-function",
        Value::Object(..) => "object", // Updated to match your 'Object' rename
    };
    Ok(Value::Literal(Literal::Str(type_name.to_string())))
}

fn native_concat(args: &[Value]) -> Result<Value, &'static str> {
    if args.is_empty() {
        return Err("concat expects at least 1 argument");
    }

    // Check if first argument is a string or list
    match &args[0] {
        Value::Literal(s) => {
            match s {
                Literal::Str(_) => {
                    let mut result = String::new();
                    for arg in args {
                        match arg {
                            Value::Literal(s) => {
                                match s {
                                    Literal::Str(s) => result.push_str(s),
                                    _ => return Err("concat on strings requires all arguments to be strings"),
                                }
                            }
                            _ => return Err("concat on strings requires all arguments to be strings"),
                        }
                    }
                    Ok(Value::Literal(Literal::Str(result)))
                }
                _ => Err("concat expects strings or lists"),
            }
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
            match &args[0] {
                Value::Literal(n) => {
                    match n {
                        Literal::Int(n_val) => {
                            if *n_val < 0 {
                                return Err("range expects non-negative integer");
                            }
                            let values: Vec<Value> = (0..*n_val)
                                .map(|i| Value::Literal(Literal::Int(i))) 
                                .collect();

                            Ok(Value::List(values))
                        }
                        _ => Err("range expects integer argument"),
                    }
                }
                _ => Err("range expects integer argument"),
            }
        }
        2 => {
            // range(start, end) -> [start, start+1, ..., end-1]
            match (&args[0], &args[1]) {
                (Value::Literal(s), Value::Literal(e)) => {
                    match(s, e) {
                        (Literal::Int(start), Literal::Int(end)) => {
                            let values: Vec<Value> = (*start..*end)
                                .map(|i| Value::Literal(Literal::Int(i)))
                                .collect();
                            Ok(Value::List(values))
                        }
                        _ => Err("range expects integer arguments"),
                    }
                }
                _ => Err("range expects integer arguments"),
            }
        }
        3 => {
            // range(start, end, step) -> [start, start+step, ..., end-step]
            match (&args[0], &args[1], &args[2]) {
                (Value::Literal(start), Value::Literal(end), Value::Literal(step)) => {
                    match (start, end, step) {
                        (Literal::Int(start), Literal::Int(end), Literal::Int(step)) => {
                            if *step == 0 {
                                return Err("range step cannot be zero");
                            }
                            let mut values = Vec::new();
                            let mut current = *start;
                            if *step > 0 {
                                while current < *end {
                                    values.push(Value::Literal(Literal::Int(current)));
                                    current += step;
                                }
                            } else {
                                while current > *end {
                                    values.push(Value::Literal(Literal::Int(current)));
                                    current += step;
                                }
                            }
                            Ok(Value::List(values))
                        }
                        _ => Err("range expects integer arguments"),
                    }
                }
                _ => Err("range expects integer arguments"),
            }
        }
        _ => Err("range expects 1, 2, or 3 arguments"),
    }
}

fn is_valid_var_name(name: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    re.is_match(name)
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

    pub fn run(&self, program: Program) -> Result<Value, &'static str> {
        self.eval_program(program, None)
    }

    pub fn eval_in_env(&self, program: Program, env: EnvRef) -> Result<Value, &'static str> {
        self.eval_with_env(program, Some(env))
    }

    fn program(&self, program: Program, env: Option<EnvRef>) -> Result<Value, &'static str> {
        let env = env.unwrap_or_else(|| self.globals.clone());
        for stmt in program.stmts {
            self.eval_stmt(stmt, Some(env.clone()))?;
        }
        Ok(Value::Literal(Literal::Null))
    }

    fn eval_stmt(&self, stmt: Stmt, env: Option<EnvRef>) -> Result<Value, &'static str> {
        let env = env.unwrap_or_else(|| self.globals.clone()); 

        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr, Some(env)),
            Stmt::Block(exprs) => self.eval_block(exprs, env),
            Stmt::Let(expr) => self.eval_let(expr, env),
            Stmt::Assign(name, expr) => self.eval_assign(name, expr, env),
            Stmt::If(condition, then_branch, else_branch) => self.eval_if(condition, then_branch, else_branch, env),
            Stmt::While(condition, body) => self.eval_while(condition, body, env),
            Stmt::Function(name, params, body) => self.eval_fnction(name, params, body, env),
            Stmt::Class(name, parent, body) => self.eval_class(name, parent, body, env),
        }
    }

    fn eval_let(&self, declarations: Vec<String, Option<Expr>>, env: EnvRef) -> Result<Value, &'static str> {
        for decl in declarations {
            let (name, expr_opt) = decl;
            if !Self::is_valid_var_name(&name) {
                return Err("invalid variable name");
            }
            let value = if let Some(expr) = expr_opt {
                self.eval_with_env(expr, Some(env.clone()))?
            } else {
                Value::Literal(Literal::Null)
            };
            env.borrow_mut().set(name, value);
        }
        Ok(Value::Literal(Literal::Null))
    }

    fn eval_assign(&self, name: String, expr: Expr, env: EnvRef) -> Result<Value, &'static str> {
        let value = self.eval_expr(expr, Some(env.clone()))?;
        env.borrow_mut().update(name.clone(), value.clone())?;
        Ok(value)
    }

    fn eval_if(
        &self,
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        let cond_value = self.eval_condition(condition, Some(env.clone()))?;

        // Determine truthiness: Null, Bool(false), Int(0), and Float(0.0) are falsy
        let is_truthy = match cond_value {
            Value::Null => false,
            Value::Literal(Literal::Bool(b)) => b,
            Value::Literal(Literal::Int(0)) => false,
            Value::Literal(Literal::Float(f)) if f == 0.0 => false,
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

    fn eval_while(
        &self,
        condition: Expr,
        body: Vec<Stmt>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        // Don't create a new scope - execute in the current environment
        // This allows loop variables to be updated
        let mut result = Value::Null;

        loop {
            let cond_value = self.eval_with_env(condition.clone(), Some(env.clone()))?;

            // Determine truthiness: Null, Bool(false), Int(0), and Float(0.0) are falsy
            let is_truthy = match cond_value {
                Value::Null => false,
                Value::Literal(Literal::Bool(b)) => b,
                Value::Literal(Literal::Int(0)) => false,
                Value::Literal(Literal::Float(f)) if f == 0.0 => false,
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

    fn eval_block(&self, exprs: Vec<Stmt>, env: EnvRef) -> Result<Value, &'static str> {
        let mut result = Value::Null; // default value for empty block
        for expr in exprs {
            result = self.eval_with_env(expr, Some(env.clone()))?;
        }
        Ok(result)
    }

    fn eval_fnction(
        &self,
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        // Validate parameter names
        for param in &params {
            if !Self::is_valid_var_name(param) {
                return Err("invalid parameter name");
            }
        }

        // Create a closure capturing the current environment
        let func_value = Value::Function(params.clone(), body.clone(), env.clone());

        // Declare the function in the current environment
        env.borrow_mut().set(name, func_value.clone());

        Ok(func_value)
    }

    fn eval_function_call(
        &self,
        name: String,
        args: Vec<Expr>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
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

    fn eval_class(&self, name: String, parent: Option<String>, body: Vec<Stmt>, env: EnvRef) -> Result<Value, &'static str> {
        let class_env = if let Some(p) = parent {
            if let Some(Value::Environment(p_env)) = self.globals.borrow().get(&p) {
                Environment::extend(p_env.clone())
            } else {
                return Err("Parent class not found");
            }
        } else {
            Environment::new()
        };
        self.globals
            .borrow_mut()
            .set(name, Value::Environment(class_env.clone()));

        for expr in body {
            match expr {
                Stmt::Expr(Expr::Set(name, expr)) => {
                    if !Self::is_valid_var_name(&name) {
                        return Err("invalid variable name");
                    }
                    let value = self.eval_with_env(*expr, Some(class_env.clone()))?;
                    // Declare class field (overrides parent field if exists)
                    class_env.borrow_mut().set(name, value.clone());
                }
                Stmt::Function(name, params, body) => {
                    let transformed = self
                        .transformer
                        .transform(Expr::Function(name, params, body));
                    self.eval_with_env(transformed, Some(class_env.clone()))?;
                }
                _ => return Err("Invalid class definition"),
    }

    fn eval_expr(&self, expr: Expr, env: Option<EnvRef>) -> Result<Value, &'static str> {
        let env = env.unwrap_or_else(|| self.globals.clone());
        match expr {
            Expr::Literal(lit) => Ok(Self::eval_literal(lit)),

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
                // Declare/create a new variable in current scope
                env.borrow_mut().set(name, value.clone());
                Ok(value)
            }
            Expr::Let(declarations) => {

                if !Self::is_valid_var_name(&name) {
                    return Err("invalid variable name");
                }
                let value = self.eval_with_env(*expr, Some(env.clone()))?;
                // Declare/create a new variable in current scope
                env.borrow_mut().set(name, value.clone());
                Ok(value)

                let mut last_value = Value::Literal(Literal::Null);
                for decl in declarations {
                    last_value = self.eval_with_env(decl, Some(env.clone()))?;
                }
                Ok(last_value)
            }
            Expr::Assign(name, expr) => {
                if !Self::is_valid_var_name(&name) {
                    return Err("invalid variable name");
                }
                let value = self.eval_with_env(*expr, Some(env.clone()))?;
                // Update existing variable (searches parent scopes)
                env.borrow_mut().update(name.clone(), value.clone())?;
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
                let transformed = self
                    .transformer
                    .transform(Expr::Function(name, params, body));
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
                let transformed = self
                    .transformer
                    .transform(Expr::For(init, condition, update, body));
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
            Expr::Class(name, parent, body) => {
                let class_env = if let Some(p) = parent {
                    if let Some(Value::Environment(p_env)) = self.globals.borrow().get(&p) {
                        Environment::extend(p_env.clone())
                    } else {
                        return Err("Parent class not found");
                    }
                } else {
                    Environment::new()
                };
                self.globals
                    .borrow_mut()
                    .set(name, Value::Environment(class_env.clone()));

                for expr in body {
                    match expr {
                        Expr::Set(name, expr) => {
                            if !Self::is_valid_var_name(&name) {
                                return Err("invalid variable name");
                            }
                            let value = self.eval_with_env(*expr, Some(class_env.clone()))?;
                            // Declare class field (overrides parent field if exists)
                            class_env.borrow_mut().set(name, value.clone());
                        }
                        Expr::Function(name, params, body) => {
                            let transformed = self
                                .transformer
                                .transform(Expr::Function(name, params, body));
                            self.eval_with_env(transformed, Some(class_env.clone()))?;
                        }
                        _ => return Err("Invalid class definition"),
                    }
                }

                Ok(Value::Null)
            }
            Expr::New(class, args) => {
                let Some(Value::Environment(class_env)) = self.globals.borrow().get(&class) else {
                    return Err("Class not found");
                };

                //let instance =  Environment::extend(class_env.clone());
                let instance = Value::Environment(Environment::extend(class_env.clone()));

                let func = class_env
                    .borrow()
                    .get("constructor")
                    .ok_or("undefined constructor")?;

                if let Value::Function(params, body, closure_env) = func {
                    if params.len() != (args.len() + 1) {
                        return Err("argument count mismatch");
                    }

                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_with_env(arg, Some(env.clone()))?);
                    }
                    arg_values.insert(0, instance.clone());

                    let func_env = Environment::extend(closure_env);
                    for (param, value) in params.iter().zip(arg_values.iter()) {
                        func_env.borrow_mut().set(param.clone(), value.clone());
                    }
                    for expr in &body {
                        self.eval_with_env(expr.clone(), Some(func_env.clone()))?;
                    }
                }

                Ok(instance)
            }
            Expr::Property(instance, name) => {
                let Some(Value::Environment(instance)) = self.globals.borrow().get(&instance)
                else {
                    return Err("Class not found");
                };
                match instance.borrow().get(&name) {
                    Some(value) => Ok(value.clone()),
                    None => Err("Property not found"),
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

    fn eval_literal(lit: Literal) -> Value {
        Value::Literal(lit)
    }

    fn eval_binary(op: Operation, left: Value, right: Value) -> Result<Value, &'static str> {
        use Operation::*;
        use Value::*;

        match (op, left, right) {
            // Int
            (Add, Int(a), Int(b)) => Ok(Int(a + b)),
            (Sub, Int(a), Int(b)) => Ok(Int(a - b)),
            (Mul, Int(a), Int(b)) => Ok(Int(a * b)),
            (Mod, Int(a), Int(b)) => Ok(Int(a % b)),
            (Div, Int(a), Int(b)) => {
                if b == 0 {
                    Err("division by zero")
                } else {
                    Ok(Int(a / b))
                }
            }
            (And, Int(a), Int(b)) => Ok(Bool(a != 0 && b != 0)),
            (Or, Int(a), Int(b)) => Ok(Bool(a != 0 || b != 0)),
            (BitwiseAnd, Int(a), Int(b)) => Ok(Int(a & b)),
            (BitwiseOr, Int(a), Int(b)) => Ok(Int(a | b)),

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
            (And, Float(a), Float(b)) => Ok(Bool(a != 0.0 && b != 0.0)),
            (Or, Float(a), Float(b)) => Ok(Bool(a != 0.0 || b != 0.0)),

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

            (And, Str(a), Str(b)) => Ok(Bool(a.len() > 0 && b.len() > 0)),
            (Or, Str(a), Str(b)) => Ok(Bool(a.len() > 0 || b.len() > 0)),

            // Logical operations for Bool
            (And, Bool(a), Bool(b)) => Ok(Bool(a && b)),
            (Or, Bool(a), Bool(b)) => Ok(Bool(a || b)),

            // Equality operations for Bool
            (Eq, Bool(a), Bool(b)) => Ok(Bool(a == b)),
            (Neq, Bool(a), Bool(b)) => Ok(Bool(a != b)),

            // Cross-type equality checks (always false for Eq, true for Neq)
            (Eq, _, _) => Ok(Bool(false)),
            (Neq, _, _) => Ok(Bool(true)),

            _ => Err("Invalid operation"),
        }
    }
}
