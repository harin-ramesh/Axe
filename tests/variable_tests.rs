use axe::{Axe, Expr, Operation, Value};

#[test]
fn set_and_get_variable() {
    let axe = Axe::new();

    axe.eval(
        Expr::Set("x".into(), Box::new(Expr::Int(42))),
    )
    .unwrap();

    let value = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(value, Value::Int(42));
}

#[test]
fn nested_expression_with_variable() {
    let axe = Axe::new();

    axe.eval(
        Expr::Set("x".into(), Box::new(Expr::Int(3))),
    )
    .unwrap();

    // (x + 2) * 4 = 20
    let expr = Expr::Binary(
        Operation::Mul,
        Box::new(Expr::Binary(
            Operation::Add,
            Box::new(Expr::Var("x".into())),
            Box::new(Expr::Int(2)),
        )),
        Box::new(Expr::Int(4)),
    );

    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn undefined_variable_fails() {
    let axe = Axe::new();
    let err = axe.eval(Expr::Var("y".into())).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn null_can_be_stored_in_variable() {
    let axe = Axe::new();
    
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Null))).unwrap();
    
    let value = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(value, Value::Null);
}
