use axe::{Axe, Condition, Expr, Operation, Value};

#[test]
fn while_basic_countdown() {
    let axe = Axe::new();

    // counter = 5
    // while (counter) {
    //     counter = counter - 1
    // }
    axe.eval(Expr::Set("counter".to_string(), Box::new(Expr::Int(5))))
        .unwrap();

    let expr = Expr::While(
        Condition::Var("counter".to_string()),
        vec![Expr::Set(
            "counter".to_string(),
            Box::new(Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Var("counter".to_string())),
                Box::new(Expr::Int(1)),
            )),
        )],
    );

    axe.eval(expr).unwrap();

    // After loop, counter should be 0
    let result = axe.eval(Expr::Var("counter".to_string())).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn while_with_comparison_condition() {
    let axe = Axe::new();

    // i = 0
    // sum = 0
    // while (i < 5) {
    //     sum = sum + i
    //     i = i + 1
    // }
    axe.eval(Expr::Set("i".to_string(), Box::new(Expr::Int(0))))
        .unwrap();
    axe.eval(Expr::Set("sum".to_string(), Box::new(Expr::Int(0))))
        .unwrap();

    let expr = Expr::While(
        Condition::Binary(
            Operation::Lt,
            Box::new(Condition::Var("i".to_string())),
            Box::new(Condition::Int(5)),
        ),
        vec![
            Expr::Set(
                "sum".to_string(),
                Box::new(Expr::Binary(
                    Operation::Add,
                    Box::new(Expr::Var("sum".to_string())),
                    Box::new(Expr::Var("i".to_string())),
                )),
            ),
            Expr::Set(
                "i".to_string(),
                Box::new(Expr::Binary(
                    Operation::Add,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Int(1)),
                )),
            ),
        ],
    );

    axe.eval(expr).unwrap();

    // sum should be 0 + 1 + 2 + 3 + 4 = 10
    let result = axe.eval(Expr::Var("sum".to_string())).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn while_never_executes() {
    let axe = Axe::new();

    // x = 0
    // while (0) {
    //     x = 10
    // }
    axe.eval(Expr::Set("x".to_string(), Box::new(Expr::Int(0))))
        .unwrap();

    let expr = Expr::While(
        Condition::Int(0),
        vec![Expr::Set("x".to_string(), Box::new(Expr::Int(10)))],
    );

    axe.eval(expr).unwrap();

    // x should still be 0 since loop never executed
    let result = axe.eval(Expr::Var("x".to_string())).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn while_with_false_condition() {
    let axe = Axe::new();

    // count = 0
    // while (false) {
    //     count = count + 1
    // }
    axe.eval(Expr::Set("count".to_string(), Box::new(Expr::Int(0))))
        .unwrap();

    let expr = Expr::While(
        Condition::Bool(false),
        vec![Expr::Set(
            "count".to_string(),
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("count".to_string())),
                Box::new(Expr::Int(1)),
            )),
        )],
    );

    axe.eval(expr).unwrap();

    let result = axe.eval(Expr::Var("count".to_string())).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn while_returns_last_expression_value() {
    let axe = Axe::new();

    // i = 3
    // result = while (i) {
    //     i = i - 1
    //     i * 10
    // }
    axe.eval(Expr::Set("i".to_string(), Box::new(Expr::Int(3))))
        .unwrap();

    let while_expr = Expr::While(
        Condition::Var("i".to_string()),
        vec![
            Expr::Set(
                "i".to_string(),
                Box::new(Expr::Binary(
                    Operation::Sub,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Int(1)),
                )),
            ),
            Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Var("i".to_string())),
                Box::new(Expr::Int(10)),
            ),
        ],
    );

    let result = axe.eval(while_expr).unwrap();
    // Last iteration: i becomes 0, then 0 * 10 = 0
    assert_eq!(result, Value::Int(0));
}

#[test]
fn while_with_nested_blocks() {
    let axe = Axe::new();

    // n = 3
    // total = 0
    // while (n > 0) {
    //     {
    //         temp = n * 2
    //         total = total + temp
    //     }
    //     n = n - 1
    // }
    axe.eval(Expr::Set("n".to_string(), Box::new(Expr::Int(3))))
        .unwrap();
    axe.eval(Expr::Set("total".to_string(), Box::new(Expr::Int(0))))
        .unwrap();

    let expr = Expr::While(
        Condition::Binary(
            Operation::Gt,
            Box::new(Condition::Var("n".to_string())),
            Box::new(Condition::Int(0)),
        ),
        vec![
            Expr::Block(vec![
                Expr::Set(
                    "temp".to_string(),
                    Box::new(Expr::Binary(
                        Operation::Mul,
                        Box::new(Expr::Var("n".to_string())),
                        Box::new(Expr::Int(2)),
                    )),
                ),
                Expr::Set(
                    "total".to_string(),
                    Box::new(Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("total".to_string())),
                        Box::new(Expr::Var("temp".to_string())),
                    )),
                ),
            ]),
            Expr::Set(
                "n".to_string(),
                Box::new(Expr::Binary(
                    Operation::Sub,
                    Box::new(Expr::Var("n".to_string())),
                    Box::new(Expr::Int(1)),
                )),
            ),
        ],
    );

    axe.eval(expr).unwrap();

    // total should be (3*2) + (2*2) + (1*2) = 6 + 4 + 2 = 12
    let result = axe.eval(Expr::Var("total".to_string())).unwrap();
    assert_eq!(result, Value::Int(12));
}

#[test]
fn while_empty_body_returns_null() {
    let axe = Axe::new();

    // while (false) { }
    let expr = Expr::While(Condition::Bool(false), vec![]);

    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn while_with_variable_modification() {
    let axe = Axe::new();

    // x = 1
    // while (x < 100) {
    //     x = x * 2
    // }
    axe.eval(Expr::Set("x".to_string(), Box::new(Expr::Int(1))))
        .unwrap();

    let expr = Expr::While(
        Condition::Binary(
            Operation::Lt,
            Box::new(Condition::Var("x".to_string())),
            Box::new(Condition::Int(100)),
        ),
        vec![Expr::Set(
            "x".to_string(),
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Int(2)),
            )),
        )],
    );

    axe.eval(expr).unwrap();

    // x should be 128 (1 -> 2 -> 4 -> 8 -> 16 -> 32 -> 64 -> 128)
    let result = axe.eval(Expr::Var("x".to_string())).unwrap();
    assert_eq!(result, Value::Int(128));
}

#[test]
fn nested_while_loops() {
    let axe = Axe::new();

    // i = 2
    // j = 0
    // total = 0
    // while (i > 0) {
    //     j = 2
    //     while (j > 0) {
    //         total = total + 1
    //         j = j - 1
    //     }
    //     i = i - 1
    // }
    axe.eval(Expr::Set("i".to_string(), Box::new(Expr::Int(2))))
        .unwrap();
    axe.eval(Expr::Set("j".to_string(), Box::new(Expr::Int(0))))
        .unwrap();
    axe.eval(Expr::Set("total".to_string(), Box::new(Expr::Int(0))))
        .unwrap();

    let expr = Expr::While(
        Condition::Binary(
            Operation::Gt,
            Box::new(Condition::Var("i".to_string())),
            Box::new(Condition::Int(0)),
        ),
        vec![
            Expr::Set("j".to_string(), Box::new(Expr::Int(2))),
            Expr::While(
                Condition::Binary(
                    Operation::Gt,
                    Box::new(Condition::Var("j".to_string())),
                    Box::new(Condition::Int(0)),
                ),
                vec![
                    Expr::Set(
                        "total".to_string(),
                        Box::new(Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var("total".to_string())),
                            Box::new(Expr::Int(1)),
                        )),
                    ),
                    Expr::Set(
                        "j".to_string(),
                        Box::new(Expr::Binary(
                            Operation::Sub,
                            Box::new(Expr::Var("j".to_string())),
                            Box::new(Expr::Int(1)),
                        )),
                    ),
                ],
            ),
            Expr::Set(
                "i".to_string(),
                Box::new(Expr::Binary(
                    Operation::Sub,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Int(1)),
                )),
            ),
        ],
    );

    axe.eval(expr).unwrap();

    // total should be 4 (outer loop runs 2 times, inner loop runs 2 times each = 2*2 = 4)
    let result = axe.eval(Expr::Var("total".to_string())).unwrap();
    assert_eq!(result, Value::Int(4));
}

#[test]
fn while_with_if_inside() {
    let axe = Axe::new();

    // i = 0
    // evens = 0
    // odds = 0
    // while (i < 5) {
    //     if (i == 0) {
    //         evens = evens + 1
    //     } else {
    //         odds = odds + 1
    //     }
    //     i = i + 1
    // }
    axe.eval(Expr::Set("i".to_string(), Box::new(Expr::Int(0))))
        .unwrap();
    axe.eval(Expr::Set("evens".to_string(), Box::new(Expr::Int(0))))
        .unwrap();
    axe.eval(Expr::Set("odds".to_string(), Box::new(Expr::Int(0))))
        .unwrap();

    let expr = Expr::While(
        Condition::Binary(
            Operation::Lt,
            Box::new(Condition::Var("i".to_string())),
            Box::new(Condition::Int(5)),
        ),
        vec![
            Expr::If(
                Condition::Binary(
                    Operation::Eq,
                    Box::new(Condition::Binary(
                        Operation::Sub,
                        Box::new(Condition::Var("i".to_string())),
                        Box::new(Condition::Binary(
                            Operation::Mul,
                            Box::new(Condition::Binary(
                                Operation::Div,
                                Box::new(Condition::Var("i".to_string())),
                                Box::new(Condition::Int(2)),
                            )),
                            Box::new(Condition::Int(2)),
                        )),
                    )),
                    Box::new(Condition::Int(0)),
                ),
                vec![Expr::Set(
                    "evens".to_string(),
                    Box::new(Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("evens".to_string())),
                        Box::new(Expr::Int(1)),
                    )),
                )],
                vec![Expr::Set(
                    "odds".to_string(),
                    Box::new(Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("odds".to_string())),
                        Box::new(Expr::Int(1)),
                    )),
                )],
            ),
            Expr::Set(
                "i".to_string(),
                Box::new(Expr::Binary(
                    Operation::Add,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Int(1)),
                )),
            ),
        ],
    );

    axe.eval(expr).unwrap();

    // evens should be 3 (0, 2, 4)
    // odds should be 2 (1, 3)
    let evens_result = axe.eval(Expr::Var("evens".to_string())).unwrap();
    let odds_result = axe.eval(Expr::Var("odds".to_string())).unwrap();
    assert_eq!(evens_result, Value::Int(3));
    assert_eq!(odds_result, Value::Int(2));
}
