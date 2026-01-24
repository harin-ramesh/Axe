use axe::{Axe, Expr, Literal, Operation, Program, Stmt, Value};

#[test]
fn let_creates_new_variable() {
    let axe = Axe::new();

    // Create a variable with Let (declaration)
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(10))))]),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program).unwrap();
    // Program returns null, but variable was created
    assert!(matches!(result, Value::Literal(Literal::Null)));
}

#[test]
fn let_overwrites_in_same_scope() {
    let axe = Axe::new();

    // Create a variable with Let, then overwrite
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(10))))]),
            Stmt::Assign("x".into(), Expr::Literal(Literal::Int(20))),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn assign_updates_existing_variable() {
    let axe = Axe::new();

    // Create a variable with Let then update with Assign
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(10))))]),
            Stmt::Assign("x".into(), Expr::Literal(Literal::Int(20))),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn assign_fails_on_undefined_variable() {
    let axe = Axe::new();

    // Assign to undefined variable should fail
    let program = Program {
        stmts: vec![Stmt::Assign(
            "undefined".into(),
            Expr::Literal(Literal::Int(10)),
        )],
    };

    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn assign_updates_parent_scope() {
    let axe = Axe::new();

    // Create global variable
    // Create function that uses Assign to update global
    // Call function
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "counter".into(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::Function(
                "increment".into(),
                vec![],
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "counter".into(),
                    Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("counter".into())),
                        Box::new(Expr::Literal(Literal::Int(1))),
                    ),
                )])),
            ),
            Stmt::Expr(Expr::Call("increment".into(), vec![])),
            Stmt::Expr(Expr::Call("increment".into(), vec![])),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn let_creates_local_variable_in_function() {
    let axe = Axe::new();

    // Create global variable
    // Function with Let creates a local variable (shadows global)
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(10))))]),
            Stmt::Function(
                "shadow".into(),
                vec![],
                Box::new(Stmt::Block(vec![
                    Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(999))))]),
                    Stmt::Expr(Expr::Var("x".into())),
                ])),
            ),
            Stmt::Expr(Expr::Call("shadow".into(), vec![])),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn assign_in_while_loop() {
    let axe = Axe::new();

    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("i".into(), Some(Expr::Literal(Literal::Int(0))))]),
            Stmt::Let(vec![("sum".into(), Some(Expr::Literal(Literal::Int(0))))]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var("i".into())),
                    Box::new(Expr::Literal(Literal::Int(5))),
                ),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign(
                        "sum".into(),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var("sum".into())),
                            Box::new(Expr::Var("i".into())),
                        ),
                    ),
                    Stmt::Assign(
                        "i".into(),
                        Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var("i".into())),
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
    let axe = Axe::new();

    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "123invalid".into(),
            Some(Expr::Literal(Literal::Int(10))),
        )])],
    };

    let err = axe.run(program).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn assign_updates_through_multiple_scopes() {
    let axe = Axe::new();

    // Create global
    // Outer function
    // Inner function using lambda
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("value".into(), Some(Expr::Literal(Literal::Int(1))))]),
            Stmt::Function(
                "outer".into(),
                vec![],
                Box::new(Stmt::Block(vec![
                    Stmt::Let(vec![(
                        "inner".into(),
                        Some(Expr::Lambda(
                            vec![],
                            Box::new(Stmt::Block(vec![Stmt::Assign(
                                "value".into(),
                                Expr::Literal(Literal::Int(100)),
                            )])),
                        )),
                    )]),
                    Stmt::Expr(Expr::Call("inner".into(), vec![])),
                ])),
            ),
            Stmt::Expr(Expr::Call("outer".into(), vec![])),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_ok());
}
