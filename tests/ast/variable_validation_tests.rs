use axe::{Axe, Expr, Literal, Program, Stmt};

#[test]
fn valid_variable_names() {
    let mut axe = Axe::new();

    // Valid names starting with letter
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(1))), None)]),
            Stmt::Let(vec![("myVar".into(), Some(Expr::Literal(Literal::Int(2))), None)]),
            Stmt::Let(vec![(
                "var123".into(),
                Some(Expr::Literal(Literal::Int(3))),
                None,
            )]),
            // Valid names starting with underscore
            Stmt::Let(vec![(
                "_private".into(),
                Some(Expr::Literal(Literal::Int(4))),
                None,
            )]),
            Stmt::Let(vec![("_".into(), Some(Expr::Literal(Literal::Int(5))), None)]),
            Stmt::Let(vec![("_123".into(), Some(Expr::Literal(Literal::Int(6))), None)]),
            // Valid names with underscores
            Stmt::Let(vec![(
                "my_var".into(),
                Some(Expr::Literal(Literal::Int(7))),
                None,
            )]),
            Stmt::Let(vec![(
                "CONSTANT_VALUE".into(),
                Some(Expr::Literal(Literal::Int(8))),
                None,
            )]),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn invalid_variable_name_starting_with_number() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "123var".into(),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_with_special_chars() {
    let mut axe = Axe::new();

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "my-var".into(),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "my.var".into(),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "my var".into(),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "my@var".into(),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_empty() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "".into(),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}
