//! Tree-walking interpreter implementation.
//!
//! This is the reference implementation for the Axe language.
//! It directly traverses the AST and executes statements/expressions.

use std::fmt;

use crate::ast::{Expr, ExprId, ExprKind, Literal, Operation, Program, Stmt, UnaryOp};
use crate::context::Context;
use crate::interner::Symbol;
use crate::transformer::Transformer;

use super::builtins::*;
use super::environment::{EnvRef, Environment};
use super::resolver::{Locals, Resolver};
use super::value::Value;

/// Signals that can arise during evaluation.
///
/// `Return` is not an error -- it's a control flow signal used to unwind
/// back to the nearest function call boundary and deliver a return value.
#[derive(Debug)]
pub enum EvalSignal {
    /// A real runtime error.
    Error(String),
    /// A return statement was executed; carries the returned value.
    Return(Value),
    /// A break statement was executed.
    Break,
    /// A continue statement was executed.
    Continue,
}

impl fmt::Display for EvalSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalSignal::Error(msg) => write!(f, "{}", msg),
            EvalSignal::Return(_) => write!(f, "return outside function"),
            EvalSignal::Break => write!(f, "break outside loop"),
            EvalSignal::Continue => write!(f, "continue outside loop"),
        }
    }
}

/// Allow `?` to automatically convert `&'static str` errors into `EvalSignal::Error`.
/// This keeps most existing error sites unchanged.
impl From<&'static str> for EvalSignal {
    fn from(s: &'static str) -> Self {
        EvalSignal::Error(s.to_string())
    }
}

/// Allow `assert_eq!(signal, "error message")` in tests.
impl PartialEq<&str> for EvalSignal {
    fn eq(&self, other: &&str) -> bool {
        match self {
            EvalSignal::Error(msg) => msg == *other,
            _ => false,
        }
    }
}

fn is_valid_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_valid_symbol_name(ctx: &Context, sym: Symbol) -> bool {
    is_valid_var_name(&ctx.resolve(sym))
}

/// Helper: run `eval_block` on a function body and catch `EvalSignal::Return`.
///
/// - If the block completes normally, return `Null` (no implicit return).
/// - If the block triggers `EvalSignal::Return(val)`, return `Ok(val)`.
/// - If the block triggers a real error, propagate it.
fn catch_return(result: Result<Value, EvalSignal>) -> Result<Value, EvalSignal> {
    match result {
        Ok(_) => Ok(Value::Literal(Literal::Null)),
        Err(EvalSignal::Return(val)) => Ok(val),
        Err(e) => Err(e),
    }
}

/// Tree-walking interpreter for the Axe language.
///
/// This interpreter directly evaluates the AST by recursively traversing
/// the tree structure. It's simpler than a bytecode VM but less efficient
/// for repeated execution of the same code.
pub struct TreeWalker<'ctx> {
    globals: EnvRef,
    transformer: Transformer,
    /// Resolved variable locations (expression ID -> depth)
    locals: Locals,
    /// Context for string interning (borrowed)
    ctx: &'ctx Context,
}

impl<'ctx> TreeWalker<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        let globals = Environment::new();

        // Initialize built-in functions and classes
        init_builtins(ctx, &globals);

        Self {
            globals,
            transformer: Transformer,
            locals: Locals::new(),
            ctx,
        }
    }

    /// Get a reference to the context.
    pub fn context(&self) -> &Context {
        self.ctx
    }

    /// Set the resolved variable locations.
    pub fn set_locals(&mut self, locals: Locals) {
        self.locals = locals;
    }

    pub fn run(&mut self, program: Program) -> Result<Value, EvalSignal> {
        // Transform the program first (desugar fn -> lambda, for -> while, etc.)
        let program = self.transformer.transform_program(program);

        // Auto-resolve variable scopes for O(1) lookups
        let resolver = Resolver::new();
        if let Ok(locals) = resolver.resolve(&program) {
            self.locals = locals;
        }
        // If resolution fails, we fall back to dynamic lookup (locals stays empty)

        self.eval_program(program, None)
    }

    /// Look up a variable, using resolved location if available.
    fn lookup_variable(&self, expr_id: ExprId, name: Symbol, env: &EnvRef) -> Option<Value> {
        if let Some(location) = self.locals.get(&expr_id) {
            // Use resolved depth for O(depth) lookup instead of O(n) hash lookup
            env.borrow().get_at(location.depth, name)
        } else {
            // Fall back to full scope chain lookup for unresolved variables
            // This handles globals and cases where the resolver wasn't run
            env.borrow().get(name)
        }
    }

    fn eval_program(&mut self, program: Program, env: Option<EnvRef>) -> Result<Value, EvalSignal> {
        let env = env.unwrap_or_else(|| self.globals.clone());
        let mut result = Value::Literal(Literal::Null);
        for stmt in program.stmts {
            result = match self.eval_stmt(stmt, Some(env.clone())) {
                Ok(val) => val,
                Err(EvalSignal::Return(_)) => {
                    return Err(EvalSignal::Error("return outside function".to_string()));
                }
                Err(e) => return Err(e),
            };
        }
        Ok(result)
    }

    fn eval_stmt(&mut self, stmt: Stmt, env: Option<EnvRef>) -> Result<Value, EvalSignal> {
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
            Stmt::Return(expr) => {
                let value = self.eval_expr(*expr, Some(env))?;
                Err(EvalSignal::Return(value))
            }
            Stmt::Break => Err(EvalSignal::Break),
            Stmt::Continue => Err(EvalSignal::Continue),
            Stmt::Import(module, imports) => self.eval_imports(module, imports, env),
        }
    }

    fn eval_imports(
        &mut self,
        module: Symbol,
        imports: Vec<Symbol>,
        _env: EnvRef,
    ) -> Result<Value, EvalSignal> {
        let module_name = self.ctx.resolve(module);
        let path = format!("{}.ax", module_name);
        let source = std::fs::read_to_string(&path)
            .map_err(|_| EvalSignal::Error(format!("failed to open module file: {}", path)))?;

        let mut parser = crate::Parser::new(&source, self.ctx);
        let program = parser.parse().map_err(|e| {
            EvalSignal::Error(format!("parse error in module '{}': {}", module_name, e))
        })?;

        // Create module interpreter sharing the same context (no clone needed!)
        let mut module_interpreter = TreeWalker::new(self.ctx);
        module_interpreter.run(program)?;

        for import in imports {
            if let Some(value) = module_interpreter.globals.borrow().get(import) {
                self.globals.borrow_mut().set(import, value.clone());
            } else {
                return Err(EvalSignal::Error(format!(
                    "import '{}' not found in module",
                    self.ctx.resolve(import)
                )));
            }
        }
        Ok(Value::Literal(Literal::Null))
    }

    fn eval_let(
        &mut self,
        declarations: Vec<(Symbol, Option<Expr>, Option<Expr>)>,
        env: EnvRef,
    ) -> Result<Value, EvalSignal> {
        for decl in declarations {
            let (name, expr_opt, expr_obj) = decl;

            if !is_valid_symbol_name(self.ctx, name) {
                return Err("invalid variable name".into());
            }

            // If there's a target object (like obj.prop = value), handle property assignment
            if let Some(obj_expr) = expr_obj {
                let obj = self.eval_expr(obj_expr, Some(env.clone()))?;
                if let Value::Object(obj_env) = obj {
                    let value = if let Some(expr) = expr_opt {
                        self.eval_expr(expr, Some(env.clone()))?
                    } else {
                        Value::Literal(Literal::Null)
                    };
                    obj_env.borrow_mut().set(name, value);
                } else {
                    return Err("Cannot assign property to non-object".into());
                }
            } else {
                // Normal variable declaration
                let value = if let Some(expr) = expr_opt {
                    self.eval_expr(expr, Some(env.clone()))?
                } else {
                    Value::Literal(Literal::Null)
                };
                env.borrow_mut().set(name, value);
            }
        }
        Ok(Value::Literal(Literal::Null))
    }

    fn eval_assign(&mut self, name: Symbol, expr: Expr, env: EnvRef) -> Result<Value, EvalSignal> {
        if !is_valid_symbol_name(self.ctx, name) {
            return Err("invalid variable name".into());
        }
        let value = self.eval_expr(expr, Some(env.clone()))?;
        env.borrow_mut()
            .update(name, value.clone())
            .map_err(EvalSignal::from)?;
        Ok(value)
    }

    fn eval_if(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, EvalSignal> {
        let cond_value = self.eval_expr(condition, Some(env.clone()))?;

        let is_truthy = match cond_value {
            Value::Literal(Literal::Null) => false,
            Value::Literal(Literal::Bool(b)) => b,
            Value::Literal(Literal::Int(0)) => false,
            Value::Literal(Literal::Float(0.0)) => false,
            _ => true,
        };

        let branch_exprs = if is_truthy { then_branch } else { else_branch };
        match branch_exprs.as_ref() {
            Stmt::Block(stmts) => self.eval_block(stmts.clone(), env.clone()),
            _ => Err("Invalid if branch".into()),
        }
    }

    fn eval_while(
        &mut self,
        condition: Expr,
        body: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, EvalSignal> {
        loop {
            let cond_value = self.eval_expr(condition.clone(), Some(env.clone()))?;

            let is_truthy = match cond_value {
                Value::Literal(Literal::Null) => false,
                Value::Literal(Literal::Bool(b)) => b,
                Value::Literal(Literal::Int(0)) => false,
                Value::Literal(Literal::Float(f)) => f != 0.0,
                _ => true,
            };

            if !is_truthy {
                break;
            }

            match body.as_ref() {
                Stmt::Block(stmts) => match self.eval_block(stmts.clone(), env.clone()) {
                    Ok(_) => {}
                    Err(EvalSignal::Break) => break,
                    Err(EvalSignal::Continue) => continue,
                    Err(e) => return Err(e),
                },
                _ => return Err("Invalid while body".into()),
            }
        }
        Ok(Value::Literal(Literal::Null))
    }

    fn eval_block(&mut self, stmts: Vec<Stmt>, parent_env: EnvRef) -> Result<Value, EvalSignal> {
        let block_env = Environment::extend(parent_env);

        let mut last_value = Value::Literal(Literal::Null);
        for stmt in stmts {
            last_value = self.eval_stmt(stmt, Some(block_env.clone()))?;
        }
        Ok(last_value)
    }

    fn eval_for(
        &mut self,
        var: Symbol,
        iterable: Expr,
        body: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, EvalSignal> {
        // Create a scope for the loop variable
        let for_scope = Environment::extend(env.clone());
        let iter_value = self.eval_expr(iterable, Some(for_scope.clone()))?;

        let items = match iter_value {
            Value::List(items) => items,
            _ => return Err("for loop requires an iterable (list)".into()),
        };

        for item in items {
            for_scope.borrow_mut().set(var, item);

            match body.as_ref() {
                Stmt::Block(stmts) => match self.eval_block(stmts.clone(), for_scope.clone()) {
                    Ok(_) => {}
                    Err(EvalSignal::Break) => break,
                    Err(EvalSignal::Continue) => continue,
                    Err(e) => return Err(e),
                },
                _ => return Err("Invalid for body".into()),
            }
        }
        Ok(Value::Literal(Literal::Null))
    }

    fn eval_class(
        &mut self,
        name: Symbol,
        parent: Option<Symbol>,
        body: Vec<Stmt>,
        env: EnvRef,
    ) -> Result<Value, EvalSignal> {
        // If there's a parent class, look it up and use its environment as base
        let class_env = if let Some(parent_name) = parent {
            let p_env = if let Some(Value::Object(penv)) = env.borrow().get(parent_name) {
                penv
            } else {
                return Err("Parent class not found".into());
            };
            Environment::extend(p_env.clone())
        } else {
            Environment::extend(env.clone())
        };

        // Store the class environment before processing methods (for self reference)
        env.borrow_mut().set(name, Value::Object(class_env.clone()));

        let self_sym = self.ctx.intern("self");
        let self_env = Environment::extend(class_env.clone());
        self_env
            .borrow_mut()
            .set(self_sym, Value::Object(self_env.clone()));

        for stmt in body {
            match stmt {
                Stmt::Let(decls) => {
                    self.eval_let(decls, class_env.clone())?;
                }
                Stmt::Function(fn_name, params, fn_body) => {
                    if params.first().is_some_and(|p| *p == self_sym) {
                        self.eval_function(fn_name, params, fn_body, self_env.clone())?;
                    } else {
                        self.eval_function(fn_name, params, fn_body, class_env.clone())?;
                    }
                }
                _ => {}
            }
        }

        Ok(Value::Literal(Literal::Null))
    }

    fn eval_function(
        &mut self,
        name: Symbol,
        params: Vec<Symbol>,
        body: Box<Stmt>,
        env: EnvRef,
    ) -> Result<Value, EvalSignal> {
        let func_value = Value::Function(params, body, env.clone());
        env.borrow_mut().set(name, func_value);
        Ok(Value::Literal(Literal::Null))
    }

    fn eval_expr(&mut self, expr: Expr, env: Option<EnvRef>) -> Result<Value, EvalSignal> {
        let env = env.unwrap_or_else(|| self.globals.clone());

        match expr.kind {
            ExprKind::Literal(lit) => Ok(Value::Literal(lit)),
            ExprKind::List(elements) => {
                let mut values = Vec::with_capacity(elements.len());
                for elem in elements {
                    values.push(self.eval_expr(elem, Some(env.clone()))?);
                }
                Ok(Value::List(values))
            }
            ExprKind::Var(name) => {
                let value = self.lookup_variable(expr.id, name, &env);
                match value {
                    Some(val) => Ok(val),
                    None => Err("undefined variable".into()),
                }
            }
            ExprKind::Binary(op, lhs, rhs) => {
                let left = self.eval_expr(*lhs, Some(env.clone()))?;
                let right = self.eval_expr(*rhs, Some(env))?;
                self.eval_binary(op, left, right)
            }
            ExprKind::Unary(op, operand) => {
                let value = self.eval_expr(*operand, Some(env.clone()))?;
                self.eval_unary(op, value)
            }
            ExprKind::Lambda(params, body) => {
                let func_value = Value::Function(params.clone(), body.clone(), env.clone());
                Ok(func_value)
            }
            ExprKind::Call(name, args) => {
                // Look up the function by name, respecting resolved scope depth
                let func = self.lookup_variable(expr.id, name, &env);

                match func {
                    Some(Value::Function(params, body, closure_env)) => {
                        if args.len() != params.len() {
                            return Err("wrong number of arguments".into());
                        }

                        let mut arg_values = Vec::with_capacity(args.len());
                        for arg in args {
                            arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                        }

                        let func_env = Environment::extend(closure_env);
                        for (param, value) in params.iter().zip(arg_values) {
                            func_env.borrow_mut().set(param.clone(), value.clone());
                        }

                        match *body {
                            Stmt::Block(stmts) => catch_return(self.eval_block(stmts, func_env)),
                            _ => self.eval_stmt(*body, Some(func_env)),
                        }
                    }
                    Some(Value::NativeFunction(_, native_fn)) => {
                        let mut arg_values = Vec::with_capacity(args.len());
                        for arg in args {
                            arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                        }
                        native_fn(self.ctx, &arg_values)
                    }
                    Some(_) => Err("not a function".into()),
                    None => Err("undefined function".into()),
                }
            }
            ExprKind::New(class_name, args) => {
                let class = self.lookup_variable(expr.id, class_name, &env);

                match class {
                    Some(Value::Object(class_env)) => {
                        let self_sym = self.ctx.intern("self");

                        // Create a new instance environment that inherits from the class
                        let instance_env = Environment::extend(class_env.clone());
                        let instance = Value::Object(Environment::extend(instance_env.clone()));

                        // Look for init method
                        let init_sym = self.ctx.intern("init");
                        if let Some(Value::Function(params, body, closure)) =
                            class_env.borrow().get(init_sym)
                        {
                            let mut arg_values = Vec::with_capacity(args.len());
                            for arg in args {
                                arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                            }
                            arg_values.insert(0, instance.clone());

                            let func_env = Environment::extend(closure);
                            for (param, value) in params.iter().zip(arg_values) {
                                func_env.borrow_mut().set(*param, value.clone());
                            }
                            func_env.borrow_mut().set(self_sym, instance.clone());

                            if let Stmt::Block(stmts) = *body {
                                catch_return(self.eval_block(stmts, func_env))?;
                            }
                        }

                        Ok(instance)
                    }
                    Some(_) => Err("not a class".into()),
                    None => Err("class not found".into()),
                }
            }
            ExprKind::Property(obj_expr, prop) => {
                let obj = self.eval_expr(*obj_expr, Some(env.clone()))?;
                match obj {
                    Value::Object(obj_env) => {
                        let value = obj_env.borrow().get(prop);
                        match value {
                            Some(value) => Ok(value.clone()),
                            None => Err("property not found".into()),
                        }
                    }
                    _ => Err("cannot access property on non-object".into()),
                }
            }
            ExprKind::MethodCall(obj_expr, method, args) => {
                let obj = self.eval_expr(*obj_expr, Some(env.clone()))?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                }
                self.eval_method_call(obj, method, arg_values)
            }
            ExprKind::StaticProperty(obj_expr, prop) => {
                let obj = self.eval_expr(*obj_expr, Some(env.clone()))?;
                match obj {
                    Value::Object(obj_env) => {
                        let value = obj_env.borrow().get(prop);
                        match value {
                            Some(value) => Ok(value.clone()),
                            None => Err("static property not found".into()),
                        }
                    }
                    _ => Err("cannot access static property on non-class".into()),
                }
            }
            ExprKind::StaticMethodCall(obj_expr, method, args) => {
                let Value::Object(class_env) = self.eval_expr(*obj_expr, Some(env.clone()))? else {
                    return Err("cannot call static method on non-class".into());
                };

                let func = class_env.borrow().get(method);
                match func {
                    Some(Value::Function(params, body, closure)) => {
                        // For static methods, don't pass self
                        if args.len() != params.len() {
                            return Err("wrong number of arguments".into());
                        }

                        let mut arg_values = Vec::with_capacity(args.len());
                        for arg in args {
                            arg_values.push(self.eval_expr(arg, Some(env.clone()))?);
                        }

                        let func_env = Environment::extend(closure);
                        for (param, value) in params.iter().zip(arg_values) {
                            func_env.borrow_mut().set(param.clone(), value.clone());
                        }

                        match *body {
                            Stmt::Block(stmts) => catch_return(self.eval_block(stmts, func_env)),
                            _ => self.eval_stmt(*body, Some(func_env)),
                        }
                    }
                    Some(_) => Err("not a method".into()),
                    None => Err("static method not found".into()),
                }
            }
        }
    }

    fn eval_method_call(
        &mut self,
        obj: Value,
        method: Symbol,
        args: Vec<Value>,
    ) -> Result<Value, EvalSignal> {
        // Get the class for this value type
        let class_sym = match &obj {
            Value::Literal(Literal::Str(_)) => self.ctx.intern("String"),
            Value::List(_) => self.ctx.intern("List"),
            Value::Object(obj_env) => {
                // For objects, look up the method directly on the object
                return self.eval_object_method_call(obj_env.clone(), method, args);
            }
            _ => return Err("cannot call method on this type".into()),
        };

        // Look up the class in globals
        let class = self.globals.borrow().get(class_sym);
        let Some(Value::Object(class_env)) = class else {
            return Err("internal error: built-in class not found".into());
        };

        // Look up the method in the class
        let func = class_env.borrow().get(method);
        match func {
            Some(Value::NativeFunction(_, native_fn)) => {
                // Call native method with self as first argument
                let mut all_args = Vec::with_capacity(args.len() + 1);
                all_args.push(obj);
                all_args.extend(args);
                native_fn(self.ctx, &all_args)
            }
            Some(Value::Function(params, body, closure)) => {
                // Call user-defined method with self as first argument
                let self_sym = self.ctx.intern("self");
                let has_self = params.first().is_some_and(|p| *p == self_sym);

                let expected_args = if has_self {
                    params.len() - 1
                } else {
                    params.len()
                };
                if args.len() != expected_args {
                    return Err("wrong number of arguments".into());
                }

                let func_env = Environment::extend(closure);
                if has_self {
                    func_env.borrow_mut().set(params[0], obj);
                }
                for (param, value) in params.iter().skip(if has_self { 1 } else { 0 }).zip(args) {
                    func_env.borrow_mut().set(*param, value.clone());
                }

                match *body {
                    Stmt::Block(stmts) => catch_return(self.eval_block(stmts, func_env)),
                    _ => self.eval_stmt(*body, Some(func_env)),
                }
            }
            Some(_) => Err("not a method".into()),
            None => Err("method not found".into()),
        }
    }

    fn eval_object_method_call(
        &mut self,
        obj_env: EnvRef,
        method: Symbol,
        args: Vec<Value>,
    ) -> Result<Value, EvalSignal> {
        let func = obj_env.borrow().get(method);
        match func {
            Some(Value::Function(params, body, closure)) => {
                // For instance methods, prepend self
                let self_sym = self.ctx.intern("self");
                let has_self = params.first().is_some_and(|p| *p == self_sym);

                let expected_args = if has_self {
                    params.len() - 1
                } else {
                    params.len()
                };
                if args.len() != expected_args {
                    return Err("wrong number of arguments".into());
                }

                let func_env = Environment::extend(closure);
                if has_self {
                    func_env
                        .borrow_mut()
                        .set(params[0], Value::Object(obj_env.clone()));
                }
                for (param, value) in params.iter().skip(if has_self { 1 } else { 0 }).zip(args) {
                    func_env.borrow_mut().set(*param, value.clone());
                }

                match *body {
                    Stmt::Block(stmts) => catch_return(self.eval_block(stmts, func_env)),
                    _ => self.eval_stmt(*body, Some(func_env)),
                }
            }
            Some(_) => Err("not a method".into()),
            None => Err("method not found".into()),
        }
    }

    fn eval_binary(&self, op: Operation, left: Value, right: Value) -> Result<Value, EvalSignal> {
        use Literal::*;
        use Operation::*;

        match (op, left, right) {
            // Integer arithmetic
            (Add, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Int(a + b))),
            (Sub, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Int(a - b))),
            (Mul, Value::Literal(Int(a)), Value::Literal(Int(b))) => Ok(Value::Literal(Int(a * b))),
            (Div, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                if b == 0 {
                    Err("division by zero".into())
                } else {
                    Ok(Value::Literal(Int(a / b)))
                }
            }
            (Mod, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                if b == 0 {
                    Err("division by zero".into())
                } else {
                    Ok(Value::Literal(Int(a % b)))
                }
            }

            // Float arithmetic
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
                    Err("division by zero".into())
                } else {
                    Ok(Value::Literal(Float(a / b)))
                }
            }

            // Integer comparison
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

            // Float comparison
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

            // Boolean operations
            (And, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a && b)))
            }
            (Or, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a || b)))
            }
            (Eq, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a == b)))
            }
            (Neq, Value::Literal(Bool(a)), Value::Literal(Bool(b))) => {
                Ok(Value::Literal(Bool(a != b)))
            }

            // String operations
            (Eq, Value::Literal(Str(a)), Value::Literal(Str(b))) => {
                Ok(Value::Literal(Bool(a == b)))
            }
            (Neq, Value::Literal(Str(a)), Value::Literal(Str(b))) => {
                Ok(Value::Literal(Bool(a != b)))
            }

            // Null operations
            (Eq, Value::Literal(Null), Value::Literal(Null)) => Ok(Value::Literal(Bool(true))),
            (Neq, Value::Literal(Null), Value::Literal(Null)) => Ok(Value::Literal(Bool(false))),
            (Eq, Value::Literal(Null), _) | (Eq, _, Value::Literal(Null)) => {
                Ok(Value::Literal(Bool(false)))
            }
            (Neq, Value::Literal(Null), _) | (Neq, _, Value::Literal(Null)) => {
                Ok(Value::Literal(Bool(true)))
            }

            // Bitwise operations on integers
            (BitwiseAnd, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Int(a & b)))
            }
            (BitwiseOr, Value::Literal(Int(a)), Value::Literal(Int(b))) => {
                Ok(Value::Literal(Int(a | b)))
            }

            // Cross-type equality returns false
            (Eq, _, _) => Ok(Value::Literal(Bool(false))),
            (Neq, _, _) => Ok(Value::Literal(Bool(true))),

            // Comparison between different types is an error
            (Gt | Lt | Gte | Lte, _, _) => Err("comparison type mismatch".into()),

            _ => Err("Invalid operation".into()),
        }
    }

    fn eval_unary(&self, op: UnaryOp, value: Value) -> Result<Value, EvalSignal> {
        use Literal::*;
        use UnaryOp::*;

        match (op, value) {
            // Unary minus: -x
            (Neg, Value::Literal(Int(n))) => Ok(Value::Literal(Int(-n))),
            (Neg, Value::Literal(Float(f))) => Ok(Value::Literal(Float(-f))),

            // Logical not: !x
            (Not, Value::Literal(Bool(b))) => Ok(Value::Literal(Bool(!b))),
            (Not, Value::Literal(Null)) => Ok(Value::Literal(Bool(true))),
            (Not, Value::Literal(Int(0))) => Ok(Value::Literal(Bool(true))),
            (Not, Value::Literal(Int(_))) => Ok(Value::Literal(Bool(false))),
            (Not, Value::Literal(Float(f))) if f == 0.0 => Ok(Value::Literal(Bool(true))),
            (Not, Value::Literal(Float(_))) => Ok(Value::Literal(Bool(false))),
            (Not, Value::Literal(Str(s))) => {
                let is_empty = self.ctx.resolve(s).is_empty();
                Ok(Value::Literal(Bool(is_empty)))
            }
            (Not, Value::List(ref items)) if items.is_empty() => Ok(Value::Literal(Bool(true))),
            (Not, Value::List(_)) => Ok(Value::Literal(Bool(false))),
            (Not, _) => Ok(Value::Literal(Bool(false))), // functions, objects are truthy

            // Bitwise invert: ~x
            (Inv, Value::Literal(Int(n))) => Ok(Value::Literal(Int(!n))),

            _ => Err("Invalid unary operation".into()),
        }
    }
}
