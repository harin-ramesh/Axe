use axe::{Axe, Expr, Literal, Operation, Program, Stmt, Value};

#[test]
fn while_basic_countdown() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "counter".to_string(),
                Some(Expr::Literal(Literal::Int(5))),
            )]),
            Stmt::While(
                Expr::Var("counter".to_string()),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "counter".to_string(),
                    Expr::Binary(
                        Operation::Sub,
                        Box::new(Expr::Var("counter".to_string())),
                        Box::new(Expr::Literal(Literal::Int(1))),
                    ),
                )])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn while_with_comparison_condition() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "i".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::Let(vec![(
                "sum".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Literal(Literal::Int(5))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign(
                        "sum".to_string(),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var("sum".to_string())),
                            Box::new(Expr::Var("i".to_string())),
                        ),
                    ),
                    Stmt::Assign(
                        "i".to_string(),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var("i".to_string())),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                ])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn while_never_executes() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::While(
                Expr::Literal(Literal::Int(0)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "x".to_string(),
                    Expr::Literal(Literal::Int(10)),
                )])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn while_with_false_condition() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "count".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::While(
                Expr::Literal(Literal::Bool(false)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "count".to_string(),
                    Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("count".to_string())),
                        Box::new(Expr::Literal(Literal::Int(1))),
                    ),
                )])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn while_with_nested_blocks() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "n".to_string(),
                Some(Expr::Literal(Literal::Int(3))),
            )]),
            Stmt::Let(vec![(
                "total".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Gt,
                    Box::new(Expr::Var("n".to_string())),
                    Box::new(Expr::Literal(Literal::Int(0))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Block(vec![
                        Stmt::Let(vec![(
                            "temp".to_string(),
                            Some(Expr::Binary(
                                Operation::Mul,
                                Box::new(Expr::Var("n".to_string())),
                                Box::new(Expr::Literal(Literal::Int(2))),
                            )),
                        )]),
                        Stmt::Assign(
                            "total".to_string(),
                            Expr::Binary(
                                Operation::Add,
                                Box::new(Expr::Var("total".to_string())),
                                Box::new(Expr::Var("temp".to_string())),
                            ),
                        ),
                    ]),
                    Stmt::Assign(
                        "n".to_string(),
                        Expr::Binary(
                            Operation::Sub,
                            Box::new(Expr::Var("n".to_string())),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                ])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn while_empty_body_returns_null() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![Stmt::While(
            Expr::Literal(Literal::Bool(false)),
            Box::new(Stmt::Block(vec![])),
        )],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Null)));
}

#[test]
fn while_with_variable_modification() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Int(1))),
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Literal(Literal::Int(100))),
                ),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "x".to_string(),
                    Expr::Binary(
                        Operation::Mul,
                        Box::new(Expr::Var("x".to_string())),
                        Box::new(Expr::Literal(Literal::Int(2))),
                    ),
                )])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn nested_while_loops() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "i".to_string(),
                Some(Expr::Literal(Literal::Int(2))),
            )]),
            Stmt::Let(vec![(
                "j".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::Let(vec![(
                "total".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Gt,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Literal(Literal::Int(0))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign("j".to_string(), Expr::Literal(Literal::Int(2))),
                    Stmt::While(
                        Expr::Binary(
                            Operation::Gt,
                            Box::new(Expr::Var("j".to_string())),
                            Box::new(Expr::Literal(Literal::Int(0))),
                        ),
                        Box::new(Stmt::Block(vec![
                            Stmt::Assign(
                                "total".to_string(),
                                Expr::Binary(
                                    Operation::Add,
                                    Box::new(Expr::Var("total".to_string())),
                                    Box::new(Expr::Literal(Literal::Int(1))),
                                ),
                            ),
                            Stmt::Assign(
                                "j".to_string(),
                                Expr::Binary(
                                    Operation::Sub,
                                    Box::new(Expr::Var("j".to_string())),
                                    Box::new(Expr::Literal(Literal::Int(1))),
                                ),
                            ),
                        ])),
                    ),
                    Stmt::Assign(
                        "i".to_string(),
                        Expr::Binary(
                            Operation::Sub,
                            Box::new(Expr::Var("i".to_string())),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                ])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn while_with_if_inside() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "i".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::Let(vec![(
                "evens".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::Let(vec![(
                "odds".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Literal(Literal::Int(5))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::If(
                        Expr::Binary(
                            Operation::Eq,
                            Box::new(Expr::Binary(
                                Operation::Mod,
                                Box::new(Expr::Var("i".to_string())),
                                Box::new(Expr::Literal(Literal::Int(2))),
                            )),
                            Box::new(Expr::Literal(Literal::Int(0))),
                        ),
                        Box::new(Stmt::Block(vec![Stmt::Assign(
                            "evens".to_string(),
                            Expr::Binary(
                                Operation::Add,
                                Box::new(Expr::Var("evens".to_string())),
                                Box::new(Expr::Literal(Literal::Int(1))),
                            ),
                        )])),
                        Box::new(Stmt::Block(vec![Stmt::Assign(
                            "odds".to_string(),
                            Expr::Binary(
                                Operation::Add,
                                Box::new(Expr::Var("odds".to_string())),
                                Box::new(Expr::Literal(Literal::Int(1))),
                            ),
                        )])),
                    ),
                    Stmt::Assign(
                        "i".to_string(),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var("i".to_string())),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                ])),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}
