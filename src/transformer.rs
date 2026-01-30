use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::{Expr, Literal, Operation, Stmt};

/// Global counter for generating unique variable names in for-loop desugaring.
/// Each nested for-loop gets unique `__iter_N`, `__idx_N`, `__len_N` names
/// to avoid collisions when loops share the same scope.
static FOR_LOOP_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
            //     let __len = __iter.len();
            //     while (__idx < __len) {
            //         let var = __iter.get(__idx);
            //         body;
            //         __idx = __idx + 1;
            //     }
            // }
            Stmt::For(var, iterable, body) => {
                // Create unique internal variable names to support nested for-loops
                let id = FOR_LOOP_COUNTER.fetch_add(1, Ordering::Relaxed);
                let iter_var = format!("__iter_{}", id);
                let idx_var = format!("__idx_{}", id);
                let len_var = format!("__len_{}", id);

                // let __iter = iterable;
                let let_iter = Stmt::Let(vec![(iter_var.clone(), Some(iterable), None)]);

                // let __idx = 0;
                let let_idx = Stmt::Let(vec![(
                    idx_var.clone(),
                    Some(Expr::Literal(Literal::Int(0))),
                    None,
                )]);

                // let __len = __iter.len();
                let len_call = Expr::MethodCall(
                    Box::new(Expr::Var(iter_var.clone())),
                    "len".to_string(),
                    vec![],
                );
                let let_len = Stmt::Let(vec![(len_var.clone(), Some(len_call), None)]);

                // __idx < __len
                let condition = Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var(idx_var.clone())),
                    Box::new(Expr::Var(len_var)),
                );

                // let var = __iter.get(__idx);
                let get_call = Expr::MethodCall(
                    Box::new(Expr::Var(iter_var)),
                    "get".to_string(),
                    vec![Expr::Var(idx_var.clone())],
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

                // while (__idx < __len) { ... }
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
