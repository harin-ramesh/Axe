#[cfg(test)]
use crate::ast::Literal;
use crate::ast::{Expr, Stmt};

/// AST Transformer - converts syntactic sugar to core forms.
///
/// Currently the only transformation is:
/// - `fn name(params) { body }` → `let name = |params| { body }`
pub struct Transformer;

impl Transformer {
    /// Transform a statement, desugaring any syntactic sugar.
    pub fn transform_stmt(&self, stmt: Stmt) -> Stmt {
        match stmt {
            // SYNTACTIC SUGAR: fn name(params) { body } -> let name = lambda(params) { body }
            Stmt::Function(name, params, body) => {
                let lambda = Expr::Lambda(params, body);
                Stmt::Let(vec![(name, Some(lambda))])
            }

            // All other statements pass through unchanged
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_to_lambda() {
        // fn add(x, y) { body }
        // Should transform to:
        // let add = lambda(x, y) { body }

        let transformer = Transformer;
        let func = Stmt::Function(
            "add".to_string(),
            vec!["x".to_string(), "y".to_string()],
            Box::new(Stmt::Expr(Expr::Var("body".to_string()))),
        );

        let transformed = transformer.transform_stmt(func);

        match transformed {
            Stmt::Let(bindings) => {
                assert_eq!(bindings.len(), 1);
                let (name, init) = &bindings[0];
                assert_eq!(name, "add");
                match init {
                    Some(Expr::Lambda(params, _body)) => {
                        assert_eq!(params, &vec!["x".to_string(), "y".to_string()]);
                    }
                    _ => panic!("Expected Lambda"),
                }
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn test_non_function_passes_through() {
        // Other statements should pass through unchanged (structurally)
        let transformer = Transformer;

        let expr_stmt = Stmt::Expr(Expr::Var("x".to_string()));
        let transformed = transformer.transform_stmt(expr_stmt);
        assert!(matches!(transformed, Stmt::Expr(Expr::Var(_))));

        let num_stmt = Stmt::Expr(Expr::Literal(Literal::Int(42)));
        let transformed = transformer.transform_stmt(num_stmt);
        assert!(matches!(
            transformed,
            Stmt::Expr(Expr::Literal(Literal::Int(42)))
        ));
    }
}
