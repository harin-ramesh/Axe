use axe::{Axe, Context};
use axe::{Expr, Literal, Operation, Program, Stmt, Value};

#[test]
fn let_creates_new_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(10))));
}

#[test]
fn let_overwrites_in_same_scope() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Assign(ctx.intern("x"), Expr::Literal(Literal::Int(20))),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(20))));
}

#[test]
fn assign_updates_existing_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Assign(ctx.intern("x"), Expr::Literal(Literal::Int(20))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn assign_fails_on_undefined_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![Stmt::Assign(
            ctx.intern("undefined"),
            Expr::Literal(Literal::Int(10)),
        )],
    };

    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn assign_updates_parent_scope() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("counter"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Function(
                ctx.intern("increment"),
                vec![],
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    ctx.intern("counter"),
                    Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var(ctx.intern("counter"))),
                        Box::new(Expr::Literal(Literal::Int(1))),
                    ),
                )])),
            ),
            Stmt::Expr(Expr::Call(ctx.intern("increment"), vec![])),
            Stmt::Expr(Expr::Call(ctx.intern("increment"), vec![])),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn let_creates_local_variable_in_function() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Function(
                ctx.intern("shadow"),
                vec![],
                Box::new(Stmt::Block(vec![
                    Stmt::Let(vec![(
                        ctx.intern("x"),
                        Some(Expr::Literal(Literal::Int(999))),
                        None,
                    )]),
                    Stmt::Expr(Expr::Var(ctx.intern("x"))),
                ])),
            ),
            Stmt::Expr(Expr::Call(ctx.intern("shadow"), vec![])),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn assign_in_while_loop() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("i"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Let(vec![(
                ctx.intern("sum"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var(ctx.intern("i"))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign(
                        ctx.intern("sum"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var(ctx.intern("sum"))),
                            Box::new(Expr::Var(ctx.intern("i"))),
                        ),
                    ),
                    Stmt::Assign(
                        ctx.intern("i"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var(ctx.intern("i"))),
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
fn let_with_invalid_name_fails() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            ctx.intern("123invalid"),
            Some(Expr::Literal(Literal::Int(10))),
            None,
        )])],
    };

    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn assign_updates_through_multiple_scopes() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("value"),
                Some(Expr::Literal(Literal::Int(1))),
                None,
            )]),
            Stmt::Function(
                ctx.intern("outer"),
                vec![],
                Box::new(Stmt::Block(vec![
                    Stmt::Let(vec![(
                        ctx.intern("inner"),
                        Some(Expr::Lambda(
                            vec![],
                            Box::new(Stmt::Block(vec![Stmt::Assign(
                                ctx.intern("value"),
                                Expr::Literal(Literal::Int(100)),
                            )])),
                        )),
                        None,
                    )]),
                    Stmt::Expr(Expr::Call(ctx.intern("inner"), vec![])),
                ])),
            ),
            Stmt::Expr(Expr::Call(ctx.intern("outer"), vec![])),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn block_creates_own_scope() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Block(vec![Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )])]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_err());
}

#[test]
fn inner_scope_can_access_outer_variables() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(42))),
                None,
            )]),
            Stmt::Let(vec![(
                ctx.intern("result"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Block(vec![Stmt::Assign(
                ctx.intern("result"),
                Expr::Var(ctx.intern("x")),
            )]),
            Stmt::Expr(Expr::Var(ctx.intern("result"))),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(42))));
}

#[test]
fn block_let_shadows_outer_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Block(vec![Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(99))),
                None,
            )])]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(10))));
}

#[test]
fn block_assign_modifies_outer_scope() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Block(vec![Stmt::Assign(
                ctx.intern("x"),
                Expr::Literal(Literal::Int(99)),
            )]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(99))));
}

#[test]
fn assign_modifies_outer_scope_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Block(vec![Stmt::Assign(
                ctx.intern("x"),
                Expr::Literal(Literal::Int(50)),
            )]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(50))));
}

#[test]
fn nested_blocks_scope_correctly() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("a"),
                Some(Expr::Literal(Literal::Int(1))),
                None,
            )]),
            Stmt::Let(vec![(
                ctx.intern("result"),
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::Block(vec![
                Stmt::Let(vec![(
                    ctx.intern("b"),
                    Some(Expr::Literal(Literal::Int(2))),
                    None,
                )]),
                Stmt::Block(vec![
                    Stmt::Let(vec![(
                        ctx.intern("c"),
                        Some(Expr::Literal(Literal::Int(3))),
                        None,
                    )]),
                    Stmt::Assign(
                        ctx.intern("result"),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Binary(
                                Operation::Add,
                                Box::new(Expr::Var(ctx.intern("a"))),
                                Box::new(Expr::Var(ctx.intern("b"))),
                            )),
                            Box::new(Expr::Var(ctx.intern("c"))),
                        ),
                    ),
                ]),
            ]),
            Stmt::Expr(Expr::Var(ctx.intern("result"))),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(6))));
}

#[test]
fn function_has_own_scope() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(100))),
                None,
            )]),
            Stmt::Function(
                ctx.intern("get_param"),
                vec![ctx.intern("x")],
                Box::new(Stmt::Block(vec![Stmt::Return(Box::new(Expr::Var(
                    ctx.intern("x"),
                )))])),
            ),
            Stmt::Expr(Expr::Call(
                ctx.intern("get_param"),
                vec![Expr::Literal(Literal::Int(5))],
            )),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(5))));
}

#[test]
fn function_scope_does_not_leak() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Function(
                ctx.intern("create_local"),
                vec![],
                Box::new(Stmt::Block(vec![Stmt::Let(vec![(
                    ctx.intern("local_var"),
                    Some(Expr::Literal(Literal::Int(999))),
                    None,
                )])])),
            ),
            Stmt::Expr(Expr::Call(ctx.intern("create_local"), vec![])),
            Stmt::Expr(Expr::Var(ctx.intern("local_var"))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_err());
}
