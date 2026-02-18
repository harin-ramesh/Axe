use axe::{Axe, Context, Expr, Literal, Operation, Program, Stmt};

#[test]
fn assign_to_existing_global_variable() {
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
fn let_creates_variable_if_not_exists() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            ctx.intern("x"),
            Some(Expr::Literal(Literal::Int(10))),
            None,
        )])],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn assign_with_invalid_name_fails() {
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
fn assign_using_expression() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(5))),
                None,
            )]),
            Stmt::Assign(
                ctx.intern("x"),
                Expr::Binary(
                    Operation::Mul,
                    Box::new(Expr::Var(ctx.intern("x"))),
                    Box::new(Expr::Literal(Literal::Int(2))),
                ),
            ),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn let_in_block_updates_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Block(vec![
                Stmt::Assign(ctx.intern("x"), Expr::Literal(Literal::Int(100))),
                Stmt::Expr(Expr::Var(ctx.intern("x"))),
            ]),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn let_in_nested_block_updates() {
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
                Expr::Literal(Literal::Int(20)),
            )]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn let_in_same_scope_updates() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(1))),
                None,
            )]),
            Stmt::Assign(ctx.intern("x"), Expr::Literal(Literal::Int(100))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn let_updates_through_blocks() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(1))),
                None,
            )]),
            Stmt::Block(vec![
                Stmt::Assign(ctx.intern("x"), Expr::Literal(Literal::Int(10))),
                Stmt::Block(vec![Stmt::Assign(
                    ctx.intern("x"),
                    Expr::Literal(Literal::Int(20)),
                )]),
            ]),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}
