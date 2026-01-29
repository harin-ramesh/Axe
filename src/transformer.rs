use crate::ast::{Expr, Literal, Operation, Stmt};

/// AST Transformer - converts syntactic sugar to core forms.
///
/// Transformations:
/// - `fn name(params) { body }` → `let name = |params| { body }`
/// - `for var in iterable { body }` → while loop with index
pub struct Transformer;

impl Transformer {
    /// Transform a statement, desugaring any syntactic sugar.
    pub fn transform_stmt(&self, stmt: Stmt) -> Stmt {
        match stmt {
            // SYNTACTIC SUGAR: fn name(params) { body } -> let name = lambda(params) { body }
            Stmt::Function(name, params, body) => {
                let lambda = Expr::Lambda(params, body);
                Stmt::Let(vec![(name, Some(lambda), None)])
            }

            // SYNTACTIC SUGAR: for var in iterable { body } -> while loop
            // Transforms to:
            // {
            //     let __iter = iterable;
            //     let __idx = 0;
            //     let __len = len(__iter);
            //     where (__idx < __len) {
            //         let var = get(__iter, __idx);
            //         body;
            //         __idx = __idx + 1;
            //     }
            // }
            Stmt::For(var, iterable, body) => {
                // Create internal variable names
                let iter_var = "__iter".to_string();
                let idx_var = "__idx".to_string();
                let len_var = "__len".to_string();

                // let __iter = iterable;
                let let_iter = Stmt::Let(vec![(iter_var.clone(), Some(iterable), None)]);

                // let __idx = 0;
                let let_idx = Stmt::Let(vec![(
                    idx_var.clone(),
                    Some(Expr::Literal(Literal::Int(0))),
                    None,
                )]);

                // let __len = len(__iter);
                let len_call = Expr::Call("len".to_string(), vec![Expr::Var(iter_var.clone())]);
                let let_len = Stmt::Let(vec![(len_var.clone(), Some(len_call), None)]);

                // __idx < __len
                let condition = Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var(idx_var.clone())),
                    Box::new(Expr::Var(len_var)),
                );

                // let var = get(__iter, __idx);
                let get_call = Expr::Call(
                    "get".to_string(),
                    vec![Expr::Var(iter_var), Expr::Var(idx_var.clone())],
                );
                let let_var = Stmt::Let(vec![(var, Some(get_call), None)]);

                // __idx = __idx + 1;
                let increment = Stmt::Assign(
                    idx_var.clone(),
                    Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var(idx_var)),
                        Box::new(Expr::Literal(Literal::Int(1))),
                    ),
                );

                // Build while body: { let var = get(...); body; __idx = __idx + 1; }
                let while_body = match *body {
                    Stmt::Block(mut stmts) => {
                        let mut new_stmts = vec![let_var];
                        new_stmts.append(&mut stmts);
                        new_stmts.push(increment);
                        Stmt::Block(new_stmts)
                    }
                    other => Stmt::Block(vec![let_var, other, increment]),
                };

                // where (__idx < __len) { ... }
                let while_stmt = Stmt::While(condition, Box::new(while_body));

                // Wrap everything in a block
                Stmt::Block(vec![let_iter, let_idx, let_len, while_stmt])
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
                let (name, init, _obj) = &bindings[0];
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
