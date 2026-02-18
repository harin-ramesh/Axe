use axe::{Axe, Context, Expr, Literal, Operation, Program, Stmt, Value};

#[test]
fn while_basic_countdown() {
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("counter"),
                Some(Expr::Literal(Literal::Int(5))),
                None,
            )]),
            Stmt::While(
                Expr::Var(context.intern("counter")),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    context.intern("counter"),
                    Expr::Binary(
                        Operation::Sub,
                        Box::new(Expr::Var(context.intern("counter"))),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("i"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("sum"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var(context.intern("i"))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign(
                        context.intern("sum"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var(context.intern("sum"))),
                            Box::new(Expr::Var(context.intern("i"))),
                        ),
                    ),
                    Stmt::Assign(
                        context.intern("i"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var(context.intern("i"))),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("x"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Literal(Literal::Int(0)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    context.intern("x"),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("count"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Literal(Literal::Bool(false)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    context.intern("count"),
                    Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var(context.intern("count"))),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("n"),
                Some(Expr::Literal(Literal::Int(3))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("total"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Gt,
                    Box::new(Expr::Var(context.intern("n"))),
                    Box::new(Expr::Literal(Literal::Int(0))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Block(vec![
                        Stmt::Let(vec![(
                            context.intern("temp"),
                            Some(Expr::Binary(
                                Operation::Mul,
                                Box::new(Expr::Var(context.intern("n"))),
                                Box::new(Expr::Literal(Literal::Int(2))),
                            )),
                            None,
                        )]),
                        Stmt::Assign(
                            context.intern("total"),
                            Expr::Binary(
                                Operation::Add,
                                Box::new(Expr::Var(context.intern("total"))),
                                Box::new(Expr::Var(context.intern("temp"))),
                            ),
                        ),
                    ]),
                    Stmt::Assign(
                        context.intern("n"),
                        Expr::Binary(
                            Operation::Sub,
                            Box::new(Expr::Var(context.intern("n"))),
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
fn while_count_1_to_10_sum_is_correct() {
    let context = Context::new();
    let mut axe = Axe::new(&context);

    // Count from 1 to 10 and sum the values
    // Expected sum: 1+2+3+4+5+6+7+8+9+10 = 55
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("i"),
                Some(Expr::Literal(Literal::Int(1))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("sum"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lte,
                    Box::new(Expr::Var(context.intern("i"))),
                    Box::new(Expr::Literal(Literal::Int(10))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign(
                        context.intern("sum"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var(context.intern("sum"))),
                            Box::new(Expr::Var(context.intern("i"))),
                        ),
                    ),
                    Stmt::Assign(
                        context.intern("i"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var(context.intern("i"))),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                ])),
            ),
            // Verify sum equals 55
            Stmt::Expr(Expr::Binary(
                Operation::Eq,
                Box::new(Expr::Var(context.intern("sum"))),
                Box::new(Expr::Literal(Literal::Int(55))),
            )),
        ],
    };

    let result = axe.run(program).unwrap();
    // The last expression (sum == 55) should evaluate to true
    assert!(matches!(result, Value::Literal(Literal::Bool(true))));
}

#[test]
fn while_empty_body_returns_null() {
    let context = Context::new();
    let mut axe = Axe::new(&context);

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
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("x"),
                Some(Expr::Literal(Literal::Int(1))),
                None,
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var(context.intern("x"))),
                    Box::new(Expr::Literal(Literal::Int(100))),
                ),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    context.intern("x"),
                    Expr::Binary(
                        Operation::Mul,
                        Box::new(Expr::Var(context.intern("x"))),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("i"),
                Some(Expr::Literal(Literal::Int(2))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("j"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("total"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Gt,
                    Box::new(Expr::Var(context.intern("i"))),
                    Box::new(Expr::Literal(Literal::Int(0))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign(context.intern("j"), Expr::Literal(Literal::Int(2))),
                    Stmt::While(
                        Expr::Binary(
                            Operation::Gt,
                            Box::new(Expr::Var(context.intern("j"))),
                            Box::new(Expr::Literal(Literal::Int(0))),
                        ),
                        Box::new(Stmt::Block(vec![
                            Stmt::Assign(
                                context.intern("total"),
                                Expr::Binary(
                                    Operation::Add,
                                    Box::new(Expr::Var(context.intern("total"))),
                                    Box::new(Expr::Literal(Literal::Int(1))),
                                ),
                            ),
                            Stmt::Assign(
                                context.intern("j"),
                                Expr::Binary(
                                    Operation::Sub,
                                    Box::new(Expr::Var(context.intern("j"))),
                                    Box::new(Expr::Literal(Literal::Int(1))),
                                ),
                            ),
                        ])),
                    ),
                    Stmt::Assign(
                        context.intern("i"),
                        Expr::Binary(
                            Operation::Sub,
                            Box::new(Expr::Var(context.intern("i"))),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("i"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("evens"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("odds"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var(context.intern("i"))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::If(
                        Expr::Binary(
                            Operation::Eq,
                            Box::new(Expr::Binary(
                                Operation::Mod,
                                Box::new(Expr::Var(context.intern("i"))),
                                Box::new(Expr::Literal(Literal::Int(2))),
                            )),
                            Box::new(Expr::Literal(Literal::Int(0))),
                        ),
                        Box::new(Stmt::Block(vec![Stmt::Assign(
                            context.intern("evens"),
                            Expr::Binary(
                                Operation::Add,
                                Box::new(Expr::Var(context.intern("evens"))),
                                Box::new(Expr::Literal(Literal::Int(1))),
                            ),
                        )])),
                        Box::new(Stmt::Block(vec![Stmt::Assign(
                            context.intern("odds"),
                            Expr::Binary(
                                Operation::Add,
                                Box::new(Expr::Var(context.intern("odds"))),
                                Box::new(Expr::Literal(Literal::Int(1))),
                            ),
                        )])),
                    ),
                    Stmt::Assign(
                        context.intern("i"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var(context.intern("i"))),
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
