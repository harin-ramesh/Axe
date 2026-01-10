use axe::{Axe, Expr, Operation, Value};

#[test]
fn empty_block_returns_null() {
    let axe = Axe::new();
    let block = Expr::Block(vec![]);
    let result = axe.eval(block).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn block_returns_last_expression() {
    let axe = Axe::new();

    // Block with multiple expressions
    let block = Expr::Block(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]);

    let result = axe.eval(block).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn block_with_variables() {
    let axe = Axe::new();

    // Block: let x = 10, let y = 20, return x + y
    let block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        Expr::Set("y".into(), Box::new(Expr::Int(20))),
        Expr::Binary(
            Operation::Add,
            Box::new(Expr::Var("x".into())),
            Box::new(Expr::Var("y".into())),
        ),
    ]);

    let result = axe.eval(block).unwrap();
    assert_eq!(result, Value::Int(30));
}
