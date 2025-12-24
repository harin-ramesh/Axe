use eva::{Condition, Eva, Expr, Operation, Parser, Value};

#[test]
fn parse_integer() {
    let mut parser = Parser::new("42").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Int(42));
}

#[test]
fn parse_float() {
    let mut parser = Parser::new("3.14").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Float(3.14));
}

#[test]
fn parse_string() {
    let mut parser = Parser::new("\"hello\"").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Str("hello".to_string()));
}

#[test]
fn parse_bool_true() {
    let mut parser = Parser::new("true").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Bool(true));
}

#[test]
fn parse_bool_false() {
    let mut parser = Parser::new("false").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Bool(false));
}

#[test]
fn parse_null() {
    let mut parser = Parser::new("null").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Null);
}

#[test]
fn parse_variable() {
    let mut parser = Parser::new("x").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Var("x".to_string()));
}

#[test]
fn parse_addition() {
    let mut parser = Parser::new("(+ 1 2)").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Binary(Operation::Add, Box::new(Expr::Int(1)), Box::new(Expr::Int(2)))
    );
}

#[test]
fn parse_nested_arithmetic() {
    let mut parser = Parser::new("(+ (* 2 3) 4)").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Binary(
            Operation::Add,
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Int(2)),
                Box::new(Expr::Int(3))
            )),
            Box::new(Expr::Int(4))
        )
    );
}

#[test]
fn parse_set() {
    let mut parser = Parser::new("(set x 10)").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Set("x".to_string(), Box::new(Expr::Int(10))));
}

#[test]
fn parse_assign() {
    let mut parser = Parser::new("(assign x 20)").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Assign("x".to_string(), Box::new(Expr::Int(20))));
}

#[test]
fn parse_comparison() {
    let mut parser = Parser::new("(> 10 5)").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Binary(Operation::Gt, Box::new(Expr::Int(10)), Box::new(Expr::Int(5)))
    );
}

#[test]
fn parse_block() {
    let mut parser = Parser::new("(block (set x 1) (+ x 2))").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![
            Expr::Set("x".to_string(), Box::new(Expr::Int(1))),
            Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Int(2))
            )
        ])
    );
}

#[test]
fn parse_if() {
    let mut parser = Parser::new("(if (> x 0) 1 2)").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::If(
            Condition::Binary(
                Operation::Gt,
                Box::new(Condition::Var("x".to_string())),
                Box::new(Condition::Int(0))
            ),
            vec![Expr::Int(1)],
            vec![Expr::Int(2)]
        )
    );
}

#[test]
fn parse_while() {
    let mut parser = Parser::new("(while (> x 0) (assign x (- x 1)))").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::While(
            Condition::Binary(
                Operation::Gt,
                Box::new(Condition::Var("x".to_string())),
                Box::new(Condition::Int(0))
            ),
            vec![Expr::Assign(
                "x".to_string(),
                Box::new(Expr::Binary(
                    Operation::Sub,
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Int(1))
                ))
            )]
        )
    );
}

#[test]
fn parse_and_eval_simple() {
    let eva = Eva::new();
    let mut parser = Parser::new("(+ 10 20)").unwrap();
    let expr = parser.parse().unwrap();
    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn parse_and_eval_with_variables() {
    let eva = Eva::new();
    
    // (set x 5)
    let mut parser = Parser::new("(set x 5)").unwrap();
    eva.eval(parser.parse().unwrap()).unwrap();
    
    // (* x 2)
    let mut parser = Parser::new("(* x 2)").unwrap();
    let result = eva.eval(parser.parse().unwrap()).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn parse_and_eval_block() {
    let eva = Eva::new();
    let input = "(block (set x 10) (set y 20) (+ x y))";
    let mut parser = Parser::new(input).unwrap();
    let expr = parser.parse().unwrap();
    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn parse_and_eval_if() {
    let eva = Eva::new();
    let input = "(if (> 10 5) \"yes\" \"no\")";
    let mut parser = Parser::new(input).unwrap();
    let expr = parser.parse().unwrap();
    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Str("yes".to_string()));
}

#[test]
fn parse_and_eval_while() {
    let eva = Eva::new();
    
    // Set initial value
    let mut parser = Parser::new("(set counter 3)").unwrap();
    eva.eval(parser.parse().unwrap()).unwrap();
    
    // Run while loop
    let input = "(while (> counter 0) (assign counter (- counter 1)))";
    let mut parser = Parser::new(input).unwrap();
    eva.eval(parser.parse().unwrap()).unwrap();
    
    // Check result
    let mut parser = Parser::new("counter").unwrap();
    let result = eva.eval(parser.parse().unwrap()).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn parse_complex_program() {
    let eva = Eva::new();
    let program = r#"
        (block
            (set sum 0)
            (set i 1)
            (while (<= i 5)
                (assign sum (+ sum i))
                (assign i (+ i 1)))
            sum)
    "#;
    
    let mut parser = Parser::new(program).unwrap();
    let expr = parser.parse().unwrap();
    let result = eva.eval(expr).unwrap();
    // sum = 1 + 2 + 3 + 4 + 5 = 15
    assert_eq!(result, Value::Int(15));
}

#[test]
fn parse_nested_if() {
    let eva = Eva::new();
    let input = "(if true (if false 1 2) 3)";
    let mut parser = Parser::new(input).unwrap();
    let expr = parser.parse().unwrap();
    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(2));
}

#[test]
fn parse_all_comparison_operators() {
    let mut parser = Parser::new("(< 1 2)").unwrap();
    parser.parse().unwrap();
    
    let mut parser = Parser::new("(> 2 1)").unwrap();
    parser.parse().unwrap();
    
    let mut parser = Parser::new("(<= 1 1)").unwrap();
    parser.parse().unwrap();
    
    let mut parser = Parser::new("(>= 2 2)").unwrap();
    parser.parse().unwrap();
    
    let mut parser = Parser::new("(== 5 5)").unwrap();
    parser.parse().unwrap();
    
    let mut parser = Parser::new("(!= 3 4)").unwrap();
    parser.parse().unwrap();
}

#[test]
fn parse_negative_numbers() {
    let mut parser = Parser::new("-42").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Int(-42));
    
    let mut parser = Parser::new("-3.14").unwrap();
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Float(-3.14));
}
