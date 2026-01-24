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

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Neg, // -x
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    List(Vec<Expr>),
    Var(String),
    Binary(Operation, Box<Expr>, Box<Expr>),
    // Unary(UnaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Lambda(Vec<String>, Box<Stmt>),
    Property(Box<Expr>, String),
    New(String, Vec<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr),
    Block(Vec<Stmt>),
    Let(Vec<(String, Option<Expr>)>),
    Assign(String, Expr),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
    Function(String, Vec<String>, Box<Stmt>),
    Class(String, Option<String>, Vec<Stmt>),
}

#[derive(Debug)]
pub enum Value {
    Literal(Literal),
    List(Vec<Value>),
    Function(Vec<String>, Box<Stmt>, EnvRef),
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
            }
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
        Value::Literal(s) => match s {
            Literal::Str(s) => Ok(Value::Literal(Literal::Int(s.len() as i64))),
            _ => Err("len expects a list or string"),
        },
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
        (Value::List(items), Value::Literal(i)) => match i {
            Literal::Int(idx) => {
                let index = if *idx < 0 {
                    (items.len() as i64 + idx) as usize
                } else {
                    *idx as usize
                };
                items.get(index).cloned().ok_or("index out of bounds")
            }
            _ => Err("get expects an integer index as second argument"),
        },
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
        Value::Literal(s) => match s {
            Literal::Str(_) => {
                let mut result = String::new();
                for arg in args {
                    match arg {
                        Value::Literal(s) => match s {
                            Literal::Str(s) => result.push_str(s),
                            _ => {
                                return Err(
                                    "concat on strings requires all arguments to be strings",
                                )
                            }
                        },
                        _ => return Err("concat on strings requires all arguments to be strings"),
                    }
                }
                Ok(Value::Literal(Literal::Str(result)))
            }
            _ => Err("concat expects strings or lists"),
        },
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
        1 => match &args[0] {
            Value::Literal(n) => match n {
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
            },
            _ => Err("range expects integer argument"),
        },
        2 => {
            // range(start, end) -> [start, start+1, ..., end-1]
            match (&args[0], &args[1]) {
                (Value::Literal(s), Value::Literal(e)) => match (s, e) {
                    (Literal::Int(start), Literal::Int(end)) => {
                        let values: Vec<Value> = (*start..*end)
                            .map(|i| Value::Literal(Literal::Int(i)))
                            .collect();
                        Ok(Value::List(values))
                    }
                    _ => Err("range expects integer arguments"),
                },
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

#[allow(dead_code)]
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

    fn eval_program(&self, program: Program, env: Option<EnvRef>) -> Result<Value, &'static str> {
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
            Stmt::If(condition, then_branch, else_branch) => {
                self.eval_if(condition, then_branch, else_branch, env)
            }
            Stmt::While(condition, body) => self.eval_while(condition, body, env),
            Stmt::Function(name, params, body) => self.eval_function(name, params, body, env),
            Stmt::Class(name, parent, body) => self.eval_class(name, parent, body, env),
        }
    }

    fn eval_let(
        &self,
        declarations: Vec<(String, Option<Expr>)>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        for decl in declarations {
            let (name, expr_opt) = decl;
            if !is_valid_var_name(&name) {
                return Err("invalid variable name");
            }
            let value = if let Some(expr) = expr_opt {
                self.eval_expr(expr, Some(env.clone()))?
            } else {
                Value::Literal(Literal::Null)
            };
            env.borrow_mut().set(name.to_string(), value);
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
        then_branch: Box<Stmt>,
        else_branch: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        let cond_value = self.eval_expr(condition, Some(env.clone()))?;

        let is_truthy = match cond_value {
            Value::Literal(Literal::Null) => false,
            Value::Literal(Literal::Bool(b)) => b,
            Value::Literal(Literal::Int(0)) => false,
            Value::Literal(Literal::Float(f)) if f == 0.0 => false,
            _ => true,
        };

        let branch_exprs = if is_truthy { then_branch } else { else_branch };
        let result = match branch_exprs.as_ref() {
            Stmt::Block(stmts) => self.eval_block(stmts.clone(), env.clone())?,
            _ => {
                return Err("Ivalid if branch");
            }
        };
        Ok(result)
    }

    fn eval_while(
        &self,
        condition: Expr,
        body: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        loop {
            let cond_value = self.eval_expr(condition.clone(), Some(env.clone()))?;

            let is_truthy = match cond_value {
                Value::Literal(Literal::Null) => false,
                Value::Literal(Literal::Bool(b)) => b,
                Value::Literal(Literal::Int(0)) => false,
                Value::Literal(Literal::Float(f)) if f == 0.0 => false,
                _ => true,
            };

            if !is_truthy {
                break;
            }

            match body.as_ref() {
                Stmt::Block(stmts) => {
                    self.eval_block(stmts.clone(), env.clone())?;
                }
                _ => {
                    return Err("Ivalid if branch");
                }
            };
        }

        Ok(Value::Literal(Literal::Null))
    }

    fn eval_block(&self, stmts: Vec<Stmt>, env: EnvRef) -> Result<Value, &'static str> {
        let mut result = Value::Literal(Literal::Null);
        for stmt in stmts {
            result = self.eval_stmt(stmt, Some(env.clone()))?;
        }
        Ok(result)
    }

    fn eval_function(
        &self,
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        for param in &params {
            if !is_valid_var_name(param) {
                return Err("invalid parameter name");
            }
        }

        let func_value = Value::Function(params.clone(), body.clone(), env.clone());
        env.borrow_mut().set(name, func_value.clone());

        Ok(func_value)
    }

    #[allow(dead_code)]
    fn eval_function_call(
        &self,
        name: String,
        args: Vec<Expr>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        let func = env.borrow().get(&name).ok_or("undefined function")?;

        match func {
            Value::Function(params, body, closure_env) => {
                if params.len() != args.len() {
                    return Err("argument count mismatch");
                }

                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                }

                let func_env = Environment::extend(closure_env);

                for (param, value) in params.iter().zip(arg_values.iter()) {
                    func_env.borrow_mut().set(param.clone(), value.clone());
                }

                let result = self.eval_block(
                    match *body {
                        Stmt::Block(stmts) => stmts,
                        _ => return Err("function body must be a block"),
                    },
                    func_env,
                )?;
                Ok(result)
            }
            Value::NativeFunction(_, native_fn) => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                }

                native_fn(&arg_values)
            }
            _ => Err("not a function"),
        }
    }

    fn eval_class(
        &self,
        name: String,
        parent: Option<String>,
        body: Vec<Stmt>,
        _env: EnvRef,
    ) -> Result<Value, &'static str> {
        let class_env = if let Some(p) = parent {
            if let Some(Value::Object(p_env)) = self.globals.borrow().get(&p) {
                Environment::extend(p_env.clone())
            } else {
                return Err("Parent class not found");
            }
        } else {
            Environment::new()
        };
        self.globals
            .borrow_mut()
            .set(name, Value::Object(class_env.clone()));

        for expr in body {
            match expr {
                Stmt::Let(decls) => {
                    self.eval_let(decls, class_env.clone())?;
                }
                Stmt::Function(name, params, body) => {
                    self.eval_function(name, params, body, class_env.clone())?;
                }
                _ => return Err("Invalid class definition"),
            }
        }

        Ok(Value::Literal(Literal::Null))
    }

    fn eval_expr(&self, expr: Expr, env: Option<EnvRef>) -> Result<Value, &'static str> {
        let env = env.unwrap_or_else(|| self.globals.clone());
        match expr {
            Expr::Literal(lit) => Ok(Self::eval_literal(lit)),

            Expr::List(elements) => {
                // Evaluate each element and create a list
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_expr(elem, Some(env.clone()))?);
                }
                Ok(Value::List(values))
            }

            Expr::Var(name) => env.borrow().get(&name).ok_or("undefined variable"),

            Expr::Binary(op, lhs, rhs) => {
                let left = self.eval_expr(*lhs, Some(env.clone()))?;
                let right = self.eval_expr(*rhs, Some(env))?;
                Self::eval_binary(op, left, right)
            }

            Expr::Lambda(params, body) => {
                // Validate parameter names
                for param in &params {
                    if is_valid_var_name(param) {
                        return Err("invalid parameter name");
                    }
                }

                // Create a closure capturing the current environment
                let func_value = Value::Function(params.clone(), body.clone(), env.clone());

                Ok(func_value)
            }

            Expr::Call(name, args) => {
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
                            arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                        }

                        // Create a new environment extending the closure environment
                        let func_env = Environment::extend(closure_env);

                        // Bind parameters to argument values
                        for (param, value) in params.iter().zip(arg_values.iter()) {
                            func_env.borrow_mut().set(param.clone(), value.clone());
                        }

                        let result = self.eval_block(
                            match *body {
                                Stmt::Block(stmts) => stmts,
                                _ => return Err("function body must be a block"),
                            },
                            func_env,
                        )?;

                        Ok(result)
                    }
                    Value::NativeFunction(_, native_fn) => {
                        // Evaluate arguments in the caller's environment
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                        }

                        // Call the native function
                        native_fn(&arg_values)
                    }
                    _ => Err("not a function"),
                }
            }
            Expr::New(class, args) => {
                let Some(Value::Object(class_env)) = self.globals.borrow().get(&class) else {
                    return Err("Class not found");
                };

                let instance = Value::Object(Environment::extend(class_env.clone()));

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
                        arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                    }
                    arg_values.insert(0, instance.clone());

                    let func_env = Environment::extend(closure_env);
                    for (param, value) in params.iter().zip(arg_values.iter()) {
                        func_env.borrow_mut().set(param.clone(), value.clone());
                    }
                    self.eval_block(
                        match *body {
                            Stmt::Block(stmts) => stmts,
                            _ => return Err("function body must be a block"),
                        },
                        func_env,
                    )?;
                }

                Ok(instance)
            }
            Expr::Property(obj_expr, name) => {
                let obj = self.eval_expr(*obj_expr, Some(env.clone()))?;

                let Value::Object(obj_env) = obj else {
                    return Err("Cannot access property on non-object");
                };

                match obj_env.borrow().get(&name) {
                    Some(value) => Ok(value.clone()),
                    None => Err("Property not found"),
                }
            }
        }
    }

    fn eval_literal(lit: Literal) -> Value {
        Value::Literal(lit)
    }

    fn eval_binary(op: Operation, left: Value, right: Value) -> Result<Value, &'static str> {
        use Literal::*;
        use Operation::*;

        match (op, left, right) {
            // Int
            (Add, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Int(a + b))),
            (Sub, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Int(a - b))),
            (Mul, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Int(a * b))),
            (Mod, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Int(a % b))),
            (Div, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                if b == 0 {
                    Err("division by zero")
                } else {
                    Ok(Value::Literal(Int(a / b)))
                }
            }
            (And, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Bool(a != 0 && b != 0)))
            }
            (Or, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Bool(a != 0 || b != 0)))
            }
            (BitwiseAnd, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Int(a & b)))
            }
            (BitwiseOr, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Int(a | b)))
            }

            // Float
            (Add, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Float(a + b)))
            }
            (Sub, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Float(a - b)))
            }
            (Mul, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Float(a * b)))
            }
            (Div, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                if b == 0.0 {
                    Err("division by zero")
                } else {
                    Ok(Value::Literal(Float(a / b)))
                }
            }
            (And, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a != 0.0 && b != 0.0)))
            }
            (Or, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a != 0.0 || b != 0.0)))
            }

            // Comparison operations for Int
            (Gt, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Bool(a > b))),
            (Lt, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Bool(a < b))),
            (Gte, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Bool(a >= b)))
            }
            (Lte, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Bool(a <= b)))
            }
            (Eq, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Bool(a == b)))
            }
            (Neq, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Bool(a != b)))
            }

            // Comparison operations for Float
            (Gt, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a > b)))
            }
            (Lt, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a < b)))
            }
            (Gte, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a >= b)))
            }
            (Lte, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a <= b)))
            }
            (Eq, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a == b)))
            }
            (Neq, Value::Literal(Float(a)), Value::Literal(Float(b))) => {
                Ok(Value::Literal(Bool(a != b)))
            }

            // Equality operations for String
            (Eq, Value::Literal(Str(ref a)), Value::Literal(Str(ref b))) => {
                Ok(Value::Literal(Bool(a == b)))
            }
            (Neq, Value::Literal(Str(ref a)), Value::Literal(Str(ref b))) => {
                Ok(Value::Literal(Bool(a != b)))
            }

            (And, Value::Literal(Str(ref a)), Value::Literal(Str(ref b))) => {
                Ok(Value::Literal(Bool(!a.is_empty() && !b.is_empty())))
            }
            (Or, Value::Literal(Str(ref a)), Value::Literal(Str(ref b))) => {
                Ok(Value::Literal(Bool(!a.is_empty() || !b.is_empty())))
            }

            // Logical operations for Bool
            (And, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a && b)))
            }
            (Or, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a || b)))
            }

            // Equality operations for Bool
            (Eq, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a == b)))
            }
            (Neq, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a != b)))
            }

            // Cross-type equality checks (always false for Eq, true for Neq)
            (Eq, _, _) => Ok(Value::Literal(Bool(false))),
            (Neq, _, _) => Ok(Value::Literal(Bool(true))),

            _ => Err("Invalid operation"),
        }
    }
}
