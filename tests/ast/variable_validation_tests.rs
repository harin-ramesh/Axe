use axe::{Axe, Context, Expr, Literal, Program, Stmt};

#[test]
fn valid_variable_names() {
    let context = Context::new();
    let mut axe = Axe::new(&context);

    // Valid names starting with letter
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("x"),
                Some(Expr::Literal(Literal::Int(1))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("myVar"),
                Some(Expr::Literal(Literal::Int(2))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("var123"),
                Some(Expr::Literal(Literal::Int(3))),
                None,
            )]),
            // Valid names starting with underscore
            Stmt::Let(vec![(
                context.intern("_private"),
                Some(Expr::Literal(Literal::Int(4))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("_"),
                Some(Expr::Literal(Literal::Int(5))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("_123"),
                Some(Expr::Literal(Literal::Int(6))),
                None,
            )]),
            // Valid names with underscores
            Stmt::Let(vec![(
                context.intern("my_var"),
                Some(Expr::Literal(Literal::Int(7))),
                None,
            )]),
            Stmt::Let(vec![(
                context.intern("CONSTANT_VALUE"),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            context.intern("123var"),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_with_special_chars() {
    let context = Context::new();
    let mut axe = Axe::new(&context);

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            context.intern("my-var"),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            context.intern("my.var"),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            context.intern("my var"),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            context.intern("my@var"),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_empty() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            context.intern(""),
            Some(Expr::Literal(Literal::Int(1))),
            None,
        )])],
    };
    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}
