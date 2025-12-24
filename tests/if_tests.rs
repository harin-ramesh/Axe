use eva::{Condition, Eva, Expr, Operation, Value};

#[test]
fn if_with_truthy_condition() {
    let eva = Eva::new();
    // if (Int(1)) { Int(10) } else { Int(20) }
    let expr = Expr::If(
        Condition::Int(1),
        vec![Expr::Int(10)],
        vec![Expr::Int(20)],
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn if_with_null_condition() {
    let eva = Eva::new();
    // if (Null) { Int(10) } else { Int(20) }
    let expr = Expr::If(
        Condition::Null,
        vec![Expr::Int(10)],
        vec![Expr::Int(20)],
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn if_with_expression_condition() {
    let eva = Eva::new();
    // if (5 + 5) { Str("yes") } else { Str("no") }
    let expr = Expr::If(
        Condition::Binary(
            Operation::Add,
            Box::new(Condition::Int(5)),
            Box::new(Condition::Int(5)),
        ),
        vec![Expr::Str("yes".to_string())],
        vec![Expr::Str("no".to_string())],
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Str("yes".to_string()));
}

#[test]
fn if_with_block_branches() {
    let eva = Eva::new();
    // if (Int(1)) { 
    //     x = 10
    //     x + 5
    // } else { 
    //     Int(0) 
    // }
    let expr = Expr::If(
        Condition::Int(1),
        vec![
            Expr::Set("x".to_string(), Box::new(Expr::Int(10))),
            Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Int(5)),
            ),
        ],
        vec![Expr::Int(0)],
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn nested_if_expressions() {
    let eva = Eva::new();
    // if (Int(1)) {
    //     if (Null) { Int(10) } else { Int(20) }
    // } else {
    //     Int(30)
    // }
    let expr = Expr::If(
        Condition::Int(1),
        vec![Expr::If(
            Condition::Null,
            vec![Expr::Int(10)],
            vec![Expr::Int(20)],
        )],
        vec![Expr::Int(30)],
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn if_with_variable_condition() {
    let eva = Eva::new();
    // x = Int(5)
    // if (x) { Str("truthy") } else { Str("falsy") }
    let setup = Expr::Set("x".to_string(), Box::new(Expr::Int(5)));
    eva.eval(setup).unwrap();

    let expr = Expr::If(
        Condition::Var("x".to_string()),
        vec![Expr::Str("truthy".to_string())],
        vec![Expr::Str("falsy".to_string())],
    );

    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Str("truthy".to_string()));
}

#[test]
fn if_non_zero_numbers_are_truthy() {
    let eva = Eva::new();
    
    // Test Int(1) is truthy
    let expr = Expr::If(
        Condition::Int(1),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(1));

    // Test negative Int is truthy
    let expr = Expr::If(
        Condition::Int(-1),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(1));

    // Test Float(1.5) is truthy
    let expr = Expr::If(
        Condition::Float(1.5),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(1));

    // Test negative Float is truthy
    let expr = Expr::If(
        Condition::Float(-0.5),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(1));

    // Test empty string is truthy
    let expr = Expr::If(
        Condition::Str("".to_string()),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(1));

    // Test non-empty string is truthy
    let expr = Expr::If(
        Condition::Str("hello".to_string()),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(1));

    // Test Bool(true) is truthy
    let expr = Expr::If(
        Condition::Bool(true),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(1));
}

#[test]
fn if_bool_true_is_truthy() {
    let eva = Eva::new();
    let expr = Expr::If(
        Condition::Bool(true),
        vec![Expr::Str("then".to_string())],
        vec![Expr::Str("else".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("then".to_string()));
}

#[test]
fn if_bool_false_is_falsy() {
    let eva = Eva::new();
    let expr = Expr::If(
        Condition::Bool(false),
        vec![Expr::Str("then".to_string())],
        vec![Expr::Str("else".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("else".to_string()));
}

#[test]
fn if_with_comparison_as_condition() {
    let eva = Eva::new();
    // if (10 > 5) { "greater" } else { "not greater" }
    let expr = Expr::If(
        Condition::Binary(
            Operation::Gt,
            Box::new(Condition::Int(10)),
            Box::new(Condition::Int(5)),
        ),
        vec![Expr::Str("greater".to_string())],
        vec![Expr::Str("not greater".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("greater".to_string()));
}

#[test]
fn if_with_equality_check() {
    let eva = Eva::new();
    // if (10 == 10) { "equal" } else { "not equal" }
    let expr = Expr::If(
        Condition::Binary(
            Operation::Eq,
            Box::new(Condition::Int(10)),
            Box::new(Condition::Int(10)),
        ),
        vec![Expr::Str("equal".to_string())],
        vec![Expr::Str("not equal".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("equal".to_string()));
}

#[test]
fn if_with_bool_variable() {
    let eva = Eva::new();
    // x = true
    // if (x) { "yes" } else { "no" }
    eva.eval(Expr::Set("x".to_string(), Box::new(Expr::Bool(true)))).unwrap();
    
    let expr = Expr::If(
        Condition::Var("x".to_string()),
        vec![Expr::Str("yes".to_string())],
        vec![Expr::Str("no".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("yes".to_string()));
}

#[test]
fn if_with_false_bool_variable() {
    let eva = Eva::new();
    // x = false
    // if (x) { "yes" } else { "no" }
    eva.eval(Expr::Set("x".to_string(), Box::new(Expr::Bool(false)))).unwrap();
    
    let expr = Expr::If(
        Condition::Var("x".to_string()),
        vec![Expr::Str("yes".to_string())],
        vec![Expr::Str("no".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("no".to_string()));
}

#[test]
fn if_falsy_values() {
    let eva = Eva::new();
    
    // Test Null is falsy
    let expr = Expr::If(
        Condition::Null,
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(2));

    // Test Bool(false) is falsy
    let expr = Expr::If(
        Condition::Bool(false),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(2));

    // Test Int(0) is falsy
    let expr = Expr::If(
        Condition::Int(0),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(2));

    // Test Float(0.0) is falsy
    let expr = Expr::If(
        Condition::Float(0.0),
        vec![Expr::Int(1)],
        vec![Expr::Int(2)],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Int(2));
}

#[test]
fn if_with_comparison_result_in_variable() {
    let eva = Eva::new();
    // result = (10 > 5)
    // if (result) { "true branch" } else { "false branch" }
    eva.eval(Expr::Set(
        "result".to_string(),
        Box::new(Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Int(10)),
            Box::new(Expr::Int(5)),
        )),
    )).unwrap();
    
    let expr = Expr::If(
        Condition::Var("result".to_string()),
        vec![Expr::Str("true branch".to_string())],
        vec![Expr::Str("false branch".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("true branch".to_string()));
}

#[test]
fn if_zero_variable_is_falsy() {
    let eva = Eva::new();
    // x = 0
    // if (x) { "truthy" } else { "falsy" }
    eva.eval(Expr::Set("x".to_string(), Box::new(Expr::Int(0)))).unwrap();
    
    let expr = Expr::If(
        Condition::Var("x".to_string()),
        vec![Expr::Str("truthy".to_string())],
        vec![Expr::Str("falsy".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("falsy".to_string()));
}

#[test]
fn if_zero_float_variable_is_falsy() {
    let eva = Eva::new();
    // x = 0.0
    // if (x) { "truthy" } else { "falsy" }
    eva.eval(Expr::Set("x".to_string(), Box::new(Expr::Float(0.0)))).unwrap();
    
    let expr = Expr::If(
        Condition::Var("x".to_string()),
        vec![Expr::Str("truthy".to_string())],
        vec![Expr::Str("falsy".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("falsy".to_string()));
}

#[test]
fn if_arithmetic_result_zero_is_falsy() {
    let eva = Eva::new();
    // if (5 - 5) { "truthy" } else { "falsy" }
    let expr = Expr::If(
        Condition::Binary(
            Operation::Sub,
            Box::new(Condition::Int(5)),
            Box::new(Condition::Int(5)),
        ),
        vec![Expr::Str("truthy".to_string())],
        vec![Expr::Str("falsy".to_string())],
    );
    assert_eq!(eva.eval(expr).unwrap(), Value::Str("falsy".to_string()));
}
