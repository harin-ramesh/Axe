#[cfg(test)]
use crate::eval::Literal;
use crate::eval::{Expr, Stmt};

/// AST Transformer - converts syntactic sugar to core forms

pub struct Transformer;

impl Transformer {
    pub fn transform_stmt(&self, stmt: Stmt) -> Stmt {
        match stmt {
            // SYNTACTIC SUGAR: fn -> lambda + let
            Stmt::Function(name, params, body) => {
                let lambda = Expr::Lambda(params, body);
                Stmt::Let(vec![(name, Some(lambda))])
            }

            // Transform nested statements in blocks
            Stmt::Block(stmts) => {
                Stmt::Block(stmts.into_iter().map(|s| self.transform_stmt(s)).collect())
            }

            // Transform if branches
            Stmt::If(cond, consequent, alternate) => Stmt::If(
                self.transform_expr(cond),
                Box::new(self.transform_stmt(*consequent)),
                Box::new(self.transform_stmt(*alternate)),
            ),

            // Transform while body
            Stmt::While(cond, body) => Stmt::While(
                self.transform_expr(cond),
                Box::new(self.transform_stmt(*body)),
            ),

            // Transform let initializers
            Stmt::Let(bindings) => Stmt::Let(
                bindings
                    .into_iter()
                    .map(|(name, init)| (name, init.map(|e| self.transform_expr(e))))
                    .collect(),
            ),

            // Transform assignment expression
            Stmt::Assign(name, expr) => Stmt::Assign(name, self.transform_expr(expr)),

            // Transform expression statement
            Stmt::Expr(expr) => Stmt::Expr(self.transform_expr(expr)),

            // Class passes through (could add transformations later)
            Stmt::Class(name, parent, methods) => Stmt::Class(
                name,
                parent,
                methods
                    .into_iter()
                    .map(|s| self.transform_stmt(s))
                    .collect(),
            ),
        }
    }

    pub fn transform_expr(&self, expr: Expr) -> Expr {
        match expr {
            // Transform binary expressions
            Expr::Binary(op, left, right) => Expr::Binary(
                op,
                Box::new(self.transform_expr(*left)),
                Box::new(self.transform_expr(*right)),
            ),

            // Transform lambda body
            Expr::Lambda(params, body) => {
                Expr::Lambda(params, Box::new(self.transform_stmt(*body)))
            }

            // Transform call arguments
            Expr::Call(name, args) => Expr::Call(
                name,
                args.into_iter().map(|e| self.transform_expr(e)).collect(),
            ),

            // Transform list elements
            Expr::List(items) => {
                Expr::List(items.into_iter().map(|e| self.transform_expr(e)).collect())
            }

            // Transform property access
            Expr::Property(obj, prop) => Expr::Property(Box::new(self.transform_expr(*obj)), prop),

            // Transform new expression arguments
            Expr::New(class, args) => Expr::New(
                class,
                args.into_iter().map(|e| self.transform_expr(e)).collect(),
            ),

            // Literals and variables pass through unchanged
            Expr::Literal(_) | Expr::Var(_) => expr,
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
