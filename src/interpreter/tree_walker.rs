//! Tree-walking interpreter implementation.
//!
//! This is the reference implementation for the Axe language.
//! It directly traverses the AST and executes statements/expressions.

use regex::Regex;

use crate::ast::{Expr, Literal, Operation, Program, Stmt, UnaryOp};
use crate::transformer::Transformer;

use super::builtins::*;
use super::environment::{EnvRef, Environment};
use super::value::Value;

fn is_valid_var_name(name: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    re.is_match(name)
}

/// Tree-walking interpreter for the Axe language.
///
/// This interpreter directly evaluates the AST by recursively traversing
/// the tree structure. It's simpler than a bytecode VM but less efficient
/// for repeated execution of the same code.
#[allow(dead_code)]
pub struct TreeWalker {
    globals: EnvRef,
    transformer: Transformer,
}

impl TreeWalker {
    pub fn new() -> Self {
        let globals = Environment::new();

        // Add built-in functions
        globals.borrow_mut().set(
            "print".to_string(),
            Value::NativeFunction("print".to_string(), native_print),
        );
        globals.borrow_mut().set(
            "type".to_string(),
            Value::NativeFunction("type".to_string(), native_type),
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
        let mut result = Value::Literal(Literal::Null);
        for stmt in program.stmts {
            result = self.eval_stmt(stmt, Some(env.clone()))?;
        }
        Ok(result)
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
            Stmt::For(var, iterable, body) => self.eval_for(var, iterable, body, env),
            Stmt::Function(name, params, body) => self.eval_function(name, params, body, env),
            Stmt::Class(name, parent, body) => self.eval_class(name, parent, body, env),
        }
    }

    fn eval_let(
        &self,
        declarations: Vec<(String, Option<Expr>, Option<Expr>)>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        for decl in declarations {
            let (name, expr_opt, expr_obj) = decl;
            if !is_valid_var_name(&name) {
                return Err("invalid variable name");
            }
            let value = if let Some(expr) = expr_opt {
                self.eval_expr(expr, Some(env.clone()))?
            } else {
                Value::Literal(Literal::Null)
            };
            if let Some(expr) = expr_obj {
                let obj = self.eval_expr(expr, Some(env.clone()))?;
                if let Value::Object(obj_env) = obj {
                    obj_env.borrow_mut().set(name.clone(), value);
                } else {
                    return Err("expected object for let ... in ...");
                }
            } else {
                env.borrow_mut().set(name.to_string(), value);
            }
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
        // Transform fn to let + lambda, then evaluate as let statement
        let transformed = self
            .transformer
            .transform_stmt(Stmt::Function(name, params, body));
        self.eval_stmt(transformed, Some(env))
    }

    fn eval_for(
        &self,
        var: String,
        iterable: Expr,
        body: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        // Transform for to while loop, then evaluate
        let transformed = self
            .transformer
            .transform_stmt(Stmt::For(var, iterable, body));
        self.eval_stmt(transformed, Some(env))
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
        env: EnvRef,
    ) -> Result<Value, &'static str> {
        let class_env = if let Some(p) = parent {
            if let Some(Value::Object(p_env)) = self.globals.borrow().get(&p) {
                Environment::extend(p_env.clone())
            } else {
                return Err("Parent class not found");
            }
        } else {
            Environment::extend(env.clone())
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
            Expr::Unary(op, operand) => {
                let value = self.eval_expr(*operand, Some(env.clone()))?;
                Self::eval_unary(op, value)
            }
            Expr::Lambda(params, body) => {
                // Validate parameter names
                for param in &params {
                    if !is_valid_var_name(param) {
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
                    .get("init")
                    .ok_or("undefined constructor")?;

                if let Value::Function(params, body, closure_env) = func {
                    if params.len() != (args.len() + 1) {
                        return Err("Argument count mismatch");
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
                    func_env.borrow_mut().set("self".to_string(), instance.clone());

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
            Expr::MethodCall(obj_expr, method, args) => {
                let obj = self.eval_expr(*obj_expr, Some(env.clone()))?;

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                }

                self.call_method(obj, &method, arg_values, env)
            }
        }
    }

    fn call_method(
        &self,
        obj: Value,
        method: &str,
        args: Vec<Value>,
        _env: EnvRef,
    ) -> Result<Value, &'static str> {
        match obj {
            // String methods
            Value::Literal(Literal::Str(ref s)) => match method {
                "len" => {
                    if !args.is_empty() {
                        return Err("len() takes no arguments");
                    }
                    Ok(Value::Literal(Literal::Int(s.len() as i64)))
                }
                "concat" => {
                    let mut result = s.clone();
                    for arg in args {
                        match arg {
                            Value::Literal(Literal::Str(other)) => result.push_str(&other),
                            _ => return Err("concat() requires string arguments"),
                        }
                    }
                    Ok(Value::Literal(Literal::Str(result)))
                }
                _ => Err("Unknown string method"),
            },

            // List methods
            Value::List(ref items) => match method {
                "len" => {
                    if !args.is_empty() {
                        return Err("len() takes no arguments");
                    }
                    Ok(Value::Literal(Literal::Int(items.len() as i64)))
                }
                "push" => {
                    if args.len() != 1 {
                        return Err("push() takes exactly 1 argument");
                    }
                    let mut new_list = items.clone();
                    new_list.push(args.into_iter().next().unwrap());
                    Ok(Value::List(new_list))
                }
                "get" => {
                    if args.len() != 1 {
                        return Err("get() takes exactly 1 argument");
                    }
                    match &args[0] {
                        Value::Literal(Literal::Int(idx)) => {
                            let index = if *idx < 0 {
                                (items.len() as i64 + idx) as usize
                            } else {
                                *idx as usize
                            };
                            items.get(index).cloned().ok_or("index out of bounds")
                        }
                        _ => Err("get() requires an integer argument"),
                    }
                }
                "concat" => {
                    let mut result = items.clone();
                    for arg in args {
                        match arg {
                            Value::List(other) => result.extend(other),
                            _ => return Err("concat() requires list arguments"),
                        }
                    }
                    Ok(Value::List(result))
                }
                _ => Err("Unknown list method"),
            },

            // Object methods - look up method in object's environment
            Value::Object(obj_env) => {
                let func = obj_env.borrow().get(method).ok_or("Method not found")?;

                match func {
                    Value::Function(params, body, closure_env) => {
                        // For object methods, first param is 'self'
                        if params.len() != args.len() + 1 {
                            return Err("argument count mismatch");
                        }

                        let func_env = Environment::extend(closure_env);

                        // Bind 'self' (first param) to the object
                        func_env
                            .borrow_mut()
                            .set(params[0].clone(), Value::Object(obj_env.clone()));

                        // Bind remaining params to arguments
                        for (param, value) in params.iter().skip(1).zip(args.iter()) {
                            func_env.borrow_mut().set(param.clone(), value.clone());
                        }
                        func_env.borrow_mut().set(
                            "self".to_string(), 
                            Value::Object(obj_env.clone())
                        );
                        self.eval_block(
                            match *body {
                                Stmt::Block(stmts) => stmts,
                                _ => return Err("function body must be a block"),
                            },
                            func_env,
                        )
                    }
                    Value::NativeFunction(_, native_fn) => native_fn(&args),
                    _ => Err("not a method"),
                }
            }

            _ => Err("Cannot call method on this type"),
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

    fn eval_unary(op: UnaryOp, operand: Value) -> Result<Value, &'static str> {
        use Literal::*;
        use UnaryOp::*;

        match (op, operand) {
            // Negation: -x
            (Neg, Value::Literal(Int(n))) => Ok(Value::Literal(Int(-n))),
            (Neg, Value::Literal(Float(f))) => Ok(Value::Literal(Float(-f))),

            // Logical not: !x (truthy/falsy)
            (Not, Value::Literal(Bool(b))) => Ok(Value::Literal(Bool(!b))),
            (Not, Value::Literal(Null)) => Ok(Value::Literal(Bool(true))),
            (Not, Value::Literal(Int(0))) => Ok(Value::Literal(Bool(true))),
            (Not, Value::Literal(Int(_))) => Ok(Value::Literal(Bool(false))),
            (Not, Value::Literal(Float(f))) if f == 0.0 => Ok(Value::Literal(Bool(true))),
            (Not, Value::Literal(Float(_))) => Ok(Value::Literal(Bool(false))),
            (Not, Value::Literal(Str(ref s))) if s.is_empty() => Ok(Value::Literal(Bool(true))),
            (Not, Value::Literal(Str(_))) => Ok(Value::Literal(Bool(false))),
            (Not, Value::List(ref items)) if items.is_empty() => Ok(Value::Literal(Bool(true))),
            (Not, Value::List(_)) => Ok(Value::Literal(Bool(false))),
            (Not, _) => Ok(Value::Literal(Bool(false))), // functions, objects are truthy

            // Bitwise invert: ~x
            (Inv, Value::Literal(Int(n))) => Ok(Value::Literal(Int(!n))),

            _ => Err("Invalid unary operation"),
        }
    }
}

impl Default for TreeWalker {
    fn default() -> Self {
        Self::new()
    }
}
