use crate::Expr;

/// AST Transformer - converts syntactic sugar to core forms

pub struct Transformer;

impl Transformer {
    pub fn transform(&self, expr: Expr) -> Expr {
        match expr {
            // SYNTACTIC SUGAR: fn -> lambda + let
            Expr::Function(name, params, body) => {
                Expr::Set(name, Box::new(Expr::Lambda(params, body)))
            }

            // SYNTACTIC SUGAR: i++ -> (i = i + 1)
            Expr::Inc(var) => Expr::Assign(
                var.clone(),
                Box::new(Expr::Binary(
                    crate::Operation::Add,
                    Box::new(Expr::Var(var)),
                    Box::new(Expr::Int(1)),
                )),
            ),

            // SYNTACTIC SUGAR: i-- -> (i = i - 1)
            Expr::Dec(var) => Expr::Assign(
                var.clone(),
                Box::new(Expr::Binary(
                    crate::Operation::Sub,
                    Box::new(Expr::Var(var)),
                    Box::new(Expr::Int(1)),
                )),
            ),

            // SYNTACTIC SUGAR: for loop -> while loop
            // (for init condition update body...) -> (begin init (while condition body... update))
            Expr::For(init, condition, update, mut body) => {
                // Add update at the end of body
                body.push(*update);

                // Create while loop
                let while_loop = Expr::While(condition, body);

                // Wrap in begin block with init
                Expr::Block(vec![*init, while_loop])
            }

            // All other expressions pass through unchanged
            // The evaluator will transform nested syntactic sugar on-demand
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_to_lambda() {
        // (fn add (x y) (+ x y))
        // Should transform to:
        // (let add (lambda (x y) (+ x y)))

        let transformer = Transformer;
        let func = Expr::Function(
            "add".to_string(),
            vec!["x".to_string(), "y".to_string()],
            vec![Expr::Var("body".to_string())], // Simplified body
        );

        let transformed = transformer.transform(func);

        match transformed {
            Expr::Set(name, lambda_box) => {
                assert_eq!(name, "add");
                match *lambda_box {
                    Expr::Lambda(params, _body) => {
                        assert_eq!(params, vec!["x".to_string(), "y".to_string()]);
                    }
                    _ => panic!("Expected Lambda"),
                }
            }
            _ => panic!("Expected Set"),
        }
    }

    #[test]
    fn test_non_function_passes_through() {
        // Other expressions should pass through unchanged
        let transformer = Transformer;
        let var = Expr::Var("x".to_string());
        let transformed = transformer.transform(var.clone());
        assert!(matches!(transformed, Expr::Var(_)));

        let num = Expr::Int(42);
        let transformed = transformer.transform(num.clone());
        assert!(matches!(transformed, Expr::Int(42)));
    }
}
