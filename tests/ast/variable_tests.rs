use axe::{Axe, Context};
use axe::{Expr, Literal, Operation, Program, Stmt};

#[test]
fn set_and_get_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(42))),
                None,
            )]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn nested_expression_with_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(3))),
                None,
            )]),
            Stmt::Expr(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Binary(
                    Operation::Add,
                    Box::new(Expr::Var(ctx.intern("x"))),
                    Box::new(Expr::Literal(Literal::Int(2))),
                )),
                Box::new(Expr::Literal(Literal::Int(4))),
            )),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn undefined_variable_fails() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Var(ctx.intern("y")))],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn null_can_be_stored_in_variable() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Null)),
                None,
            )]),
            Stmt::Expr(Expr::Var(ctx.intern("x"))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}
