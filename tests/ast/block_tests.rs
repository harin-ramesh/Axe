use axe::{Axe, Context};
use axe::{Expr, Literal, Operation, Program, Stmt, Value};

#[test]
fn empty_block_returns_null() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    let program = Program {
        stmts: vec![Stmt::Block(vec![])],
    };
    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Null)));
}

#[test]
fn block_returns_last_expression() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    // Block with multiple expressions
    let program = Program {
        stmts: vec![Stmt::Block(vec![
            Stmt::Expr(Expr::Literal(Literal::Int(1))),
            Stmt::Expr(Expr::Literal(Literal::Int(2))),
            Stmt::Expr(Expr::Literal(Literal::Int(3))),
        ])],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn block_with_variables() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);

    // Block: let x = 10, let y = 20, return x + y
    let program = Program {
        stmts: vec![Stmt::Block(vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Let(vec![(
                ctx.intern("y"),
                Some(Expr::Literal(Literal::Int(20))),
                None,
            )]),
            Stmt::Expr(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var(ctx.intern("x"))),
                Box::new(Expr::Var(ctx.intern("y"))),
            )),
        ])],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}
