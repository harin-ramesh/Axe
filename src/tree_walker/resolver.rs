//! Variable resolution pass.
//!
//! This module performs static analysis to resolve variable references
//! to their defining scope. This avoids the need for runtime scope chain
//! traversal when looking up variables.

use fxhash::FxHashMap;
use smallvec::SmallVec;

use crate::ast::{Expr, ExprId, ExprKind, Program, Stmt};
use crate::interner::Symbol;

/// Resolved location of a variable.
/// `depth` is how many scopes up to find the variable (0 = current scope).
#[derive(Debug, Clone, Copy)]
pub struct ResolvedLocation {
    pub depth: usize,
}

/// Side table mapping expression IDs to their resolved locations.
/// Uses FxHashMap for efficient lookup with potentially sparse ExprIds.
#[derive(Debug, Clone)]
pub struct Locals {
    data: FxHashMap<ExprId, ResolvedLocation>,
}

impl Locals {
    pub fn new() -> Self {
        Self {
            data: FxHashMap::default(),
        }
    }

    /// Insert a resolved location for an expression ID.
    #[inline]
    pub fn insert(&mut self, expr_id: ExprId, location: ResolvedLocation) {
        self.data.insert(expr_id, location);
    }

    /// Get the resolved location for an expression ID.
    #[inline]
    pub fn get(&self, expr_id: &ExprId) -> Option<&ResolvedLocation> {
        self.data.get(expr_id)
    }

    /// Check if the locals map is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get an iterator over all resolved locations (for testing).
    pub fn values(&self) -> impl Iterator<Item = &ResolvedLocation> {
        self.data.values()
    }
}

impl Default for Locals {
    fn default() -> Self {
        Self::new()
    }
}

/// A scope that maps variable names (as Symbols) to their declaration status.
/// Uses SmallVec for better cache locality and to avoid heap allocation for
/// small scopes (most scopes have fewer than 8 variables).
/// true = defined (ready to use), false = declared but not yet defined.
#[derive(Debug, Clone)]
struct Scope {
    vars: SmallVec<[(Symbol, bool); 8]>,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: SmallVec::new(),
        }
    }

    /// Insert or update a variable's status.
    fn insert(&mut self, name: Symbol, defined: bool) {
        // Check if variable already exists
        for (n, d) in &mut self.vars {
            if *n == name {
                *d = defined;
                return;
            }
        }
        // Not found, add new entry
        self.vars.push((name, defined));
    }

    /// Check if a variable exists in this scope.
    fn contains(&self, name: Symbol) -> bool {
        self.vars.iter().any(|(n, _)| *n == name)
    }

    /// Get a variable's defined status.
    fn get(&self, name: Symbol) -> Option<bool> {
        self.vars.iter().find(|(n, _)| *n == name).map(|(_, d)| *d)
    }
}

/// Tracks what kind of context we're resolving in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
}

/// Variable resolver that performs static scope analysis.
pub struct Resolver {
    /// Stack of scopes, each scope maps variable names to their declaration status.
    scopes: Vec<Scope>,
    /// The resolved locations for each variable/assignment expression.
    locals: Locals,
    /// Current function type (for tracking method context).
    current_function: FunctionType,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            locals: Locals::new(),
            current_function: FunctionType::None,
        }
    }

    /// Resolve all variable references in a program.
    /// Returns a mapping of expression IDs to their resolved depths.
    pub fn resolve(mut self, program: &Program) -> Result<Locals, String> {
        for stmt in &program.stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(self.locals)
    }

    /// Begin a new scope.
    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// End the current scope.
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declare a variable in the current scope (but not yet defined).
    fn declare(&mut self, name: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, false);
        }
    }

    /// Define a variable in the current scope (mark as ready to use).
    fn define(&mut self, name: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, true);
        }
    }

    /// Resolve a local variable by walking up the scope chain.
    fn resolve_local(&mut self, expr_id: ExprId, name: Symbol) {
        // Walk from innermost scope outward
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains(name) {
                self.locals.insert(expr_id, ResolvedLocation { depth: i });
                return;
            }
        }
        // If not found in any scope, it's either a global or undefined.
        // We don't add it to locals - the interpreter will fall back to global lookup.
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                for s in stmts {
                    self.resolve_stmt(s)?;
                }
                self.end_scope();
            }

            Stmt::Let(declarations) => {
                for (name, initializer, target_obj) in declarations {
                    self.declare(*name);
                    if let Some(expr) = initializer {
                        self.resolve_expr(expr)?;
                    }
                    if let Some(obj_expr) = target_obj {
                        self.resolve_expr(obj_expr)?;
                    }
                    self.define(*name);
                }
            }

            Stmt::Assign(_name, expr) => {
                self.resolve_expr(expr)?;
                // Note: Stmt::Assign doesn't have an ExprId, so we can't resolve
                // the assignment target statically. The interpreter uses
                // Environment::update which walks the scope chain at runtime.
                // To optimize this, we'd need to add an ID to Stmt::Assign.
            }

            Stmt::If(condition, then_branch, else_branch) => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                self.resolve_stmt(else_branch)?;
            }

            Stmt::While(condition, body) => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }

            Stmt::For(var, iterable, body) => {
                // Note: The interpreter evaluates the iterable expression inside the
                // for-loop's scope (see eval_for), so we must begin the scope BEFORE
                // resolving the iterable to ensure variable depths are correct.
                self.begin_scope();
                self.resolve_expr(iterable)?;
                self.declare(*var);
                self.define(*var);
                self.resolve_stmt(body)?;
                self.end_scope();
            }

            Stmt::Function(name, params, body) => {
                self.declare(*name);
                self.define(*name);
                self.resolve_function(params, body, FunctionType::Function)?;
            }

            Stmt::Class(name, parent, body) => {
                self.declare(*name);
                self.define(*name);

                // If there's a parent class, resolve it
                if let Some(parent_name) = parent {
                    // Check that class doesn't inherit from itself
                    if *parent_name == *name {
                        return Err("A class cannot inherit from itself".to_string());
                    }
                }

                // Resolve methods within the class
                for stmt in body {
                    if let Stmt::Function(_, params, method_body) = stmt {
                        self.resolve_function(params, method_body, FunctionType::Method)?;
                    } else {
                        self.resolve_stmt(stmt)?;
                    }
                }
            }

            Stmt::Return(expr) => {
                self.resolve_expr(expr)?;
            }

            Stmt::Expr(expr) => {
                self.resolve_expr(expr)?;
            }

            Stmt::Break | Stmt::Continue => {}

            Stmt::Import(_, _) => {
                // Imports are handled at runtime
            }
        }
        Ok(())
    }

    fn resolve_function(
        &mut self,
        params: &[Symbol],
        body: &Stmt,
        function_type: FunctionType,
    ) -> Result<(), String> {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        self.begin_scope();

        for param in params {
            self.declare(*param);
            self.define(*param);
        }
        self.resolve_stmt(body)?;
        self.end_scope();

        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match &expr.kind {
            ExprKind::Var(name) => {
                // Check for reading variable in its own initializer
                if let Some(scope) = self.scopes.last()
                    && let Some(defined) = scope.get(*name)
                    && !defined
                {
                    return Err("Can't read local variable in its own initializer".to_string());
                }
                self.resolve_local(expr.id, *name);
            }

            ExprKind::Binary(_, lhs, rhs) => {
                self.resolve_expr(lhs)?;
                self.resolve_expr(rhs)?;
            }

            ExprKind::Unary(_, operand) => {
                self.resolve_expr(operand)?;
            }

            ExprKind::Call(name, args) => {
                // Resolve the function name itself
                self.resolve_local(expr.id, *name);
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }

            ExprKind::Lambda(params, body) => {
                self.resolve_function(params, body, FunctionType::Function)?;
            }

            ExprKind::List(elements) => {
                for elem in elements {
                    self.resolve_expr(elem)?;
                }
            }

            ExprKind::New(_, args) => {
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }

            ExprKind::Property(obj, _) => {
                self.resolve_expr(obj)?;
            }

            ExprKind::MethodCall(obj, _, args) => {
                self.resolve_expr(obj)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }

            ExprKind::StaticProperty(obj, _) => {
                self.resolve_expr(obj)?;
            }

            ExprKind::StaticMethodCall(obj, _, args) => {
                self.resolve_expr(obj)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }

            ExprKind::Literal(_) => {}
        }
        Ok(())
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::transformer::Transformer;
    use crate::Parser;

    fn resolve_source(source: &str) -> Result<Locals, String> {
        let ctx = Context::new();
        let mut parser = Parser::new(source, &ctx);
        let program = parser.parse().map_err(|e| e.to_string())?;
        // Transform before resolving (same as interpreter does)
        let transformer = Transformer;
        let program = transformer.transform_program(program);
        let resolver = Resolver::new();
        resolver.resolve(&program)
    }

    #[test]
    fn test_resolve_local_variable() {
        // Variables declared at top level are globals, not resolved to locals.
        // Use a function to create a scope.
        let source = r#"
            fn test() {
                let x = 10;
                x;
            }
        "#;
        let locals = resolve_source(source).expect("Resolution should succeed");
        // The variable reference should be resolved within function scope
        assert!(!locals.is_empty());
    }

    #[test]
    fn test_resolve_nested_functions() {
        // Test closure capturing variable from outer function.
        let source = r#"
            fn outer() {
                let x = 10;
                fn inner() {
                    x;
                }
            }
        "#;
        let locals = resolve_source(source).expect("Resolution should succeed");
        // 'x' should be resolved from inner function at depth 2:
        // inner's block scope (0) -> inner's param scope (1) -> outer's block scope (2) where x is defined
        let has_depth_2 = locals.values().any(|loc| loc.depth == 2);
        assert!(has_depth_2, "Should resolve 'x' from closure at depth 2");
    }

    #[test]
    fn test_resolve_function_params() {
        let source = r#"
            fn add(a, b) {
                a;
            }
        "#;
        let locals = resolve_source(source).expect("Resolution should succeed");
        // Parameter 'a' is at depth 1 within function body:
        // block scope (0) -> param scope (1) where 'a' is defined
        let has_depth_1 = locals.values().any(|loc| loc.depth == 1);
        assert!(
            has_depth_1,
            "Should resolve 'a' at depth 1 (block scope -> param scope)"
        );
    }

    #[test]
    fn test_error_on_use_in_initializer() {
        // Use in function scope to test the error
        let source = r#"
            fn test() {
                let x = x;
            }
        "#;
        let result = resolve_source(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("own initializer"));
    }

    #[test]
    fn test_class_cannot_inherit_from_itself() {
        let source = r#"
            class Foo : Foo {
            }
        "#;
        let result = resolve_source(source);
        // This check may fail at parse time or resolve time depending on implementation
        // For now, just verify it doesn't succeed silently
        if result.is_ok() {
            panic!("Expected error for class inheriting from itself");
        }
    }
}
