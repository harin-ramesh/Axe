use eva::{Eva, Expr, Operation, Value};

#[test]
fn eval_addition() {
    let eva = Eva::new();
    let expr = Expr::Binary(
        Operation::Add,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(5)),
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn eval_float_addition() {
    let eva = Eva::new();
    let expr = Expr::Binary(
        Operation::Add,
        Box::new(Expr::Float(2.0)),
        Box::new(Expr::Float(0.5)),
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Float(2.5));
}

#[test]
fn division_by_zero_fails() {
    let eva = Eva::new();
    let expr = Expr::Binary(
        Operation::Div,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(0)),
    );

    let err = eva.eval(expr).unwrap_err();
    assert_eq!(err, "division by zero");
}

#[test]
fn eval_null() {
    let eva = Eva::new();
    let result = eva.eval(Expr::Null).unwrap();
    assert_eq!(result, Value::Null);
}
