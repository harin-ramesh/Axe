use axe::{Condition, Axe, Expr, Operation, Value};

// Greater than tests
#[test]
fn gt_int_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Gt,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn gt_int_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Gt,
        Box::new(Expr::Int(5)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

#[test]
fn gt_float_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Gt,
        Box::new(Expr::Float(10.5)),
        Box::new(Expr::Float(5.5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

// Less than tests
#[test]
fn lt_int_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Lt,
        Box::new(Expr::Int(5)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn lt_int_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Lt,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

// Greater than or equal tests
#[test]
fn gte_int_true_greater() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Gte,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn gte_int_true_equal() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Gte,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn gte_int_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Gte,
        Box::new(Expr::Int(5)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

// Less than or equal tests
#[test]
fn lte_int_true_less() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Lte,
        Box::new(Expr::Int(5)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn lte_int_true_equal() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Lte,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn lte_int_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Lte,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

// Equality tests
#[test]
fn eq_int_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn eq_int_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

#[test]
fn eq_float_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Float(10.5)),
        Box::new(Expr::Float(10.5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn eq_string_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Str("hello".to_string())),
        Box::new(Expr::Str("hello".to_string())),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn eq_string_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Str("hello".to_string())),
        Box::new(Expr::Str("world".to_string())),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

#[test]
fn eq_bool_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Bool(true)),
        Box::new(Expr::Bool(true)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn eq_null_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Null),
        Box::new(Expr::Null),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

// Not equal tests
#[test]
fn neq_int_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Neq,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(5)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn neq_int_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Neq,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(10)),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

// Cross-type comparisons
#[test]
fn eq_cross_type_false() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Str("10".to_string())),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(false));
}

#[test]
fn neq_cross_type_true() {
    let axe = Axe::new();
    let expr = Expr::Binary(
        Operation::Neq,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Str("10".to_string())),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

// Comparison with variables
#[test]
fn comparison_with_variables() {
    let axe = Axe::new();
    axe.eval(Expr::Set("x".to_string(), Box::new(Expr::Int(10)))).unwrap();
    axe.eval(Expr::Set("y".to_string(), Box::new(Expr::Int(5)))).unwrap();

    let expr = Expr::Binary(
        Operation::Gt,
        Box::new(Expr::Var("x".to_string())),
        Box::new(Expr::Var("y".to_string())),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

// Comparison in if expressions
#[test]
fn comparison_in_if_condition() {
    let axe = Axe::new();
    // if (10 > 5) { "yes" } else { "no" }
    let expr = Expr::If(
        Condition::Binary(
            Operation::Gt,
            Box::new(Condition::Int(10)),
            Box::new(Condition::Int(5)),
        ),
        vec![Expr::Str("yes".to_string())],
        vec![Expr::Str("no".to_string())],
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Str("yes".to_string()));
}

#[test]
fn comparison_false_in_if_condition() {
    let axe = Axe::new();
    // if (5 > 10) { "yes" } else { "no" }
    let expr = Expr::If(
        Condition::Binary(
            Operation::Gt,
            Box::new(Condition::Int(5)),
            Box::new(Condition::Int(10)),
        ),
        vec![Expr::Str("yes".to_string())],
        vec![Expr::Str("no".to_string())],
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Str("no".to_string()));
}

// Boolean values in if expressions
#[test]
fn bool_true_in_if_condition() {
    let axe = Axe::new();
    let expr = Expr::If(
        Condition::Bool(true),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Int(1));
}

#[test]
fn bool_false_in_if_condition() {
    let axe = Axe::new();
    let expr = Expr::If(
        Condition::Bool(false),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Int(2));
}

// Nested comparisons
#[test]
fn nested_comparison_expressions() {
    let axe = Axe::new();
    // (10 > 5) == (3 < 7)  -> true == true -> true
    let expr = Expr::Binary(
        Operation::Eq,
        Box::new(Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Int(10)),
            Box::new(Expr::Int(5)),
        )),
        Box::new(Expr::Binary(
            Operation::Lt,
            Box::new(Expr::Int(3)),
            Box::new(Expr::Int(7)),
        )),
    );
    assert_eq!(axe.eval(expr).unwrap(), Value::Bool(true));
}

#[test]
fn comparison_type_error() {
    let axe = Axe::new();
    // Can't use > on strings
    let expr = Expr::Binary(
        Operation::Gt,
        Box::new(Expr::Str("hello".to_string())),
        Box::new(Expr::Str("world".to_string())),
    );
    assert_eq!(axe.eval(expr).unwrap_err(), "type error");
}
