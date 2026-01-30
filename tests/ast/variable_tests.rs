use axe::{Axe, Expr, Literal, Operation, Program, Stmt};

#[test]
fn set_and_get_variable() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(42))), None)]),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn nested_expression_with_variable() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(3))), None)]),
            Stmt::Expr(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Binary(
                    Operation::Add,
                    Box::new(Expr::Var("x".into())),
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
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Var("y".into()))],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn null_can_be_stored_in_variable() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Null)), None)]),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}
