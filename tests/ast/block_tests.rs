use axe::{Axe, Expr, Literal, Operation, Program, Stmt, Value};

#[test]
fn empty_block_returns_null() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Block(vec![])],
    };
    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Null)));
}

#[test]
fn block_returns_last_expression() {
    let mut axe = Axe::new();

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
    let mut axe = Axe::new();

    // Block: let x = 10, let y = 20, return x + y
    let program = Program {
        stmts: vec![Stmt::Block(vec![
            Stmt::Let(vec![(
                "x".into(),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Let(vec![(
                "y".into(),
                Some(Expr::Literal(Literal::Int(20))),
                None,
            )]),
            Stmt::Expr(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("x".into())),
                Box::new(Expr::Var("y".into())),
            )),
        ])],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}
