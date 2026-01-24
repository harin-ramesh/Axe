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
    // Program returns the last expression's value, which is x = 10
    assert!(matches!(result, Value::Literal(Literal::Int(10))));
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

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(20))));
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

#[test]
fn block_shares_parent_scope() {
    let axe = Axe::new();

    // Variable declared in block is visible outside (blocks share parent scope)
    let program = Program {
        stmts: vec![
            Stmt::Block(vec![Stmt::Let(vec![(
                "x".into(),
                Some(Expr::Literal(Literal::Int(10))),
            )])]),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(10))));
}

#[test]
fn inner_scope_can_access_outer_variables() {
    let axe = Axe::new();

    // Inner block can read variable from outer scope
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(42))))]),
            Stmt::Block(vec![Stmt::Expr(Expr::Var("x".into()))]),
        ],
    };

    let result = axe.run(program).unwrap();
    assert!(matches!(result, Value::Literal(Literal::Int(42))));
}

#[test]
fn block_let_overwrites_outer_variable() {
    let axe = Axe::new();

    // Block let overwrites outer variable (blocks share parent scope)
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(10))))]),
            Stmt::Block(vec![Stmt::Let(vec![(
                "x".into(),
                Some(Expr::Literal(Literal::Int(99))),
            )])]),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program).unwrap();
    // x is now 99 because block shares the same scope
    assert!(matches!(result, Value::Literal(Literal::Int(99))));
}

#[test]
fn block_let_modifies_same_scope() {
    let axe = Axe::new();

    // Block let modifies same scope (no isolation)
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(10))))]),
            Stmt::Block(vec![Stmt::Let(vec![(
                "x".into(),
                Some(Expr::Literal(Literal::Int(99))),
            )])]),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program).unwrap();
    // x is 99 because blocks share the same scope
    assert!(matches!(result, Value::Literal(Literal::Int(99))));
}

#[test]
fn assign_modifies_outer_scope_variable() {
    let axe = Axe::new();

    // Assign (not let) in inner block modifies outer variable
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(10))))]),
            Stmt::Block(vec![Stmt::Assign(
                "x".into(),
                Expr::Literal(Literal::Int(50)),
            )]),
            Stmt::Expr(Expr::Var("x".into())),
        ],
    };

    let result = axe.run(program).unwrap();
    // Outer x was modified to 50
    assert!(matches!(result, Value::Literal(Literal::Int(50))));
}

#[test]
fn nested_blocks_scope_correctly() {
    let axe = Axe::new();

    // Multiple levels of nesting
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("a".into(), Some(Expr::Literal(Literal::Int(1))))]),
            Stmt::Block(vec![
                Stmt::Let(vec![("b".into(), Some(Expr::Literal(Literal::Int(2))))]),
                Stmt::Block(vec![
                    Stmt::Let(vec![("c".into(), Some(Expr::Literal(Literal::Int(3))))]),
                    Stmt::Expr(Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Binary(
                            Operation::Add,
                            Box::new(Expr::Var("a".into())),
                            Box::new(Expr::Var("b".into())),
                        )),
                        Box::new(Expr::Var("c".into())),
                    )),
                ]),
            ]),
        ],
    };

    let result = axe.run(program).unwrap();
    // 1 + 2 + 3 = 6
    assert!(matches!(result, Value::Literal(Literal::Int(6))));
}

#[test]
fn function_has_own_scope() {
    let axe = Axe::new();

    // Function parameters are in function's scope
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".into(), Some(Expr::Literal(Literal::Int(100))))]),
            Stmt::Function(
                "get_param".into(),
                vec!["x".into()],
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Var("x".into()))])),
            ),
            Stmt::Expr(Expr::Call(
                "get_param".into(),
                vec![Expr::Literal(Literal::Int(5))],
            )),
        ],
    };

    let result = axe.run(program).unwrap();
    // Function returns its parameter x=5, not the global x=100
    assert!(matches!(result, Value::Literal(Literal::Int(5))));
}

#[test]
fn function_scope_does_not_leak() {
    let axe = Axe::new();

    // Variable declared inside function is not visible outside
    let program = Program {
        stmts: vec![
            Stmt::Function(
                "create_local".into(),
                vec![],
                Box::new(Stmt::Block(vec![Stmt::Let(vec![(
                    "local_var".into(),
                    Some(Expr::Literal(Literal::Int(999))),
                )])])),
            ),
            Stmt::Expr(Expr::Call("create_local".into(), vec![])),
            Stmt::Expr(Expr::Var("local_var".into())),
        ],
    };

    let result = axe.run(program);
    assert!(result.is_err());
}
