use crate::ast::{Expr, ExprKind, Program, Stmt};

/// AST Transformer - converts syntactic sugar to core forms.
///
/// Transformations:
/// - `fn name(params) { body }` → `let name = |params| { body }`
/// - `for var in iterable { body }` → recursively transforms body
pub struct Transformer;

impl Transformer {
    /// Transform a statement, desugaring any syntactic sugar.
    pub fn transform_stmt(&self, stmt: Stmt) -> Stmt {
        match stmt {
            // SYNTACTIC SUGAR: fn name(params) { body } -> let name = lambda(params) { body }
            Stmt::Function(name, params, body) => {
                let body = Box::new(self.transform_stmt(*body));
                let lambda = Expr::Lambda(params, body);
                Stmt::Let(vec![(name, Some(lambda), None)])
            }

            // For loops: recursively transform the body
            Stmt::For(var, iterable, body) => Stmt::For(
                var,
                self.transform_expr(iterable),
                Box::new(self.transform_stmt(*body)),
            ),

            // Recursively transform nested statements to find functions/for loops
            Stmt::Block(stmts) => {
                Stmt::Block(stmts.into_iter().map(|s| self.transform_stmt(s)).collect())
            }
            Stmt::If(cond, then_branch, else_branch) => Stmt::If(
                cond,
                Box::new(self.transform_stmt(*then_branch)),
                Box::new(self.transform_stmt(*else_branch)),
            ),
            Stmt::While(cond, body) => Stmt::While(cond, Box::new(self.transform_stmt(*body))),
            Stmt::Class(name, parent, body) => Stmt::Class(
                name,
                parent,
                body.into_iter().map(|s| self.transform_stmt(s)).collect(),
            ),

            // Pass through unchanged
            other => other,
        }
    }

    /// Transform an expression, only handling lambdas which may contain functions/for loops.
    pub fn transform_expr(&self, expr: Expr) -> Expr {
        match expr.kind {
            ExprKind::Lambda(params, body) => {
                Expr::Lambda(params, Box::new(self.transform_stmt(*body)))
            }
            // All other expressions pass through unchanged
            _ => expr,
        }
    }

    /// Transform an entire program, desugaring all syntactic sugar.
    pub fn transform_program(&self, program: Program) -> Program {
        Program {
            stmts: program
                .stmts
                .into_iter()
                .map(|s| self.transform_stmt(s))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ExprKind, Literal};
    use crate::context::Context;

    #[test]
    fn test_function_to_lambda() {
        // fn add(x, y) { body }
        // Should transform to:
        // let add = lambda(x, y) { body }

        let ctx = Context::new();
        let add_sym = ctx.intern("add");
        let x_sym = ctx.intern("x");
        let y_sym = ctx.intern("y");
        let body_sym = ctx.intern("body");

        let transformer = Transformer;
        let func = Stmt::Function(
            add_sym,
            vec![x_sym, y_sym],
            Box::new(Stmt::Expr(Expr::Var(body_sym))),
        );

        let transformed = transformer.transform_stmt(func);

        match transformed {
            Stmt::Let(bindings) => {
                assert_eq!(bindings.len(), 1);
                let (name, init, _obj) = &bindings[0];
                assert_eq!(*name, add_sym);
                match init {
                    Some(expr) => match &expr.kind {
                        ExprKind::Lambda(params, _body) => {
                            assert_eq!(params, &vec![x_sym, y_sym]);
                        }
                        _ => panic!("Expected Lambda"),
                    },
                    None => panic!("Expected Some"),
                }
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn test_non_function_passes_through() {
        // Other statements should pass through unchanged (structurally)
        let ctx = Context::new();
        let x_sym = ctx.intern("x");

        let transformer = Transformer;

        let expr_stmt = Stmt::Expr(Expr::Var(x_sym));
        let transformed = transformer.transform_stmt(expr_stmt);
        assert!(matches!(
            transformed,
            Stmt::Expr(Expr {
                kind: ExprKind::Var(_),
                ..
            })
        ));

        let num_stmt = Stmt::Expr(Expr::Literal(Literal::Int(42)));
        let transformed = transformer.transform_stmt(num_stmt);
        assert!(matches!(
            transformed,
            Stmt::Expr(Expr {
                kind: ExprKind::Literal(Literal::Int(42)),
                ..
            })
        ));
    }
}
