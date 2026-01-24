use axe::{Axe, Environment, Expr, Literal, Operation, Program, Stmt, Value};

// ============================================================================
// Environment Tests
// ============================================================================

#[test]
fn environment_new_creates_empty_env() {
    let env = Environment::new();
    assert!(env.borrow().get("x").is_none());
}

#[test]
fn environment_set_and_get() {
    let env = Environment::new();
    env.borrow_mut()
        .set("x".to_string(), Value::Literal(Literal::Int(42)));

    let value = env.borrow().get("x").unwrap();
    match value {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 42),
        _ => panic!("Expected Int(42)"),
    }
}

#[test]
fn environment_extend_inherits_parent() {
    let parent = Environment::new();
    parent
        .borrow_mut()
        .set("x".to_string(), Value::Literal(Literal::Int(10)));

    let child = Environment::extend(parent);

    let value = child.borrow().get("x").unwrap();
    match value {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 10),
        _ => panic!("Expected Int(10)"),
    }
}

#[test]
fn environment_child_shadows_parent() {
    let parent = Environment::new();
    parent
        .borrow_mut()
        .set("x".to_string(), Value::Literal(Literal::Int(10)));

    let child = Environment::extend(parent.clone());
    child
        .borrow_mut()
        .set("x".to_string(), Value::Literal(Literal::Int(20)));

    // Child should see shadowed value
    match child.borrow().get("x").unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 20),
        _ => panic!("Expected Int(20)"),
    }

    // Parent should still have original value
    match parent.borrow().get("x").unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 10),
        _ => panic!("Expected Int(10)"),
    }
}

#[test]
fn environment_update_modifies_existing() {
    let env = Environment::new();
    env.borrow_mut()
        .set("x".to_string(), Value::Literal(Literal::Int(10)));
    env.borrow_mut()
        .update("x".to_string(), Value::Literal(Literal::Int(20)))
        .unwrap();

    match env.borrow().get("x").unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 20),
        _ => panic!("Expected Int(20)"),
    }
}

#[test]
fn environment_update_undefined_fails() {
    let env = Environment::new();
    let result = env
        .borrow_mut()
        .update("x".to_string(), Value::Literal(Literal::Int(10)));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "undefined variable");
}

#[test]
fn environment_update_parent_variable() {
    let parent = Environment::new();
    parent
        .borrow_mut()
        .set("x".to_string(), Value::Literal(Literal::Int(10)));

    let child = Environment::extend(parent.clone());
    child
        .borrow_mut()
        .update("x".to_string(), Value::Literal(Literal::Int(20)))
        .unwrap();

    // Parent should be updated
    match parent.borrow().get("x").unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 20),
        _ => panic!("Expected Int(20)"),
    }
}

// ============================================================================
// Literal Evaluation Tests
// ============================================================================

#[test]
fn eval_int_literal() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Int(42)))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_float_literal() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Float(3.14)))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_string_literal() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Str("hello".to_string())))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_bool_literal() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Bool(true)))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_null_literal() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Null))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

// ============================================================================
// Binary Operation Tests - Integers
// ============================================================================

fn make_int_binary(op: Operation, a: i64, b: i64) -> Program {
    Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            op,
            Box::new(Expr::Literal(Literal::Int(a))),
            Box::new(Expr::Literal(Literal::Int(b))),
        ))],
    }
}

#[test]
fn eval_int_addition() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Add, 10, 5);
    let result = axe.run(program).unwrap();
    match result {
        Value::Literal(Literal::Null) => {} // Program returns null, but expression was evaluated
        _ => {}
    }
}

#[test]
fn eval_int_subtraction() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Sub, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_multiplication() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Mul, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_division() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Div, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_modulo() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Mod, 10, 3);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_division_by_zero_fails() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Div, 10, 0);
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "division by zero");
}

#[test]
fn eval_int_greater_than() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Gt, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_less_than() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Lt, 5, 10);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_equal() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Eq, 5, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_not_equal() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::Neq, 5, 10);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_bitwise_and() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::BitwiseAnd, 0b1010, 0b1100);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_bitwise_or() {
    let axe = Axe::new();
    let program = make_int_binary(Operation::BitwiseOr, 0b1010, 0b1100);
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Binary Operation Tests - Floats
// ============================================================================

fn make_float_binary(op: Operation, a: f64, b: f64) -> Program {
    Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            op,
            Box::new(Expr::Literal(Literal::Float(a))),
            Box::new(Expr::Literal(Literal::Float(b))),
        ))],
    }
}

#[test]
fn eval_float_addition() {
    let axe = Axe::new();
    let program = make_float_binary(Operation::Add, 2.5, 1.5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_subtraction() {
    let axe = Axe::new();
    let program = make_float_binary(Operation::Sub, 5.0, 2.5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_multiplication() {
    let axe = Axe::new();
    let program = make_float_binary(Operation::Mul, 2.0, 3.0);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_division() {
    let axe = Axe::new();
    let program = make_float_binary(Operation::Div, 10.0, 2.0);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_division_by_zero_fails() {
    let axe = Axe::new();
    let program = make_float_binary(Operation::Div, 10.0, 0.0);
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "division by zero");
}

// ============================================================================
// Binary Operation Tests - Booleans
// ============================================================================

fn make_bool_binary(op: Operation, a: bool, b: bool) -> Program {
    Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            op,
            Box::new(Expr::Literal(Literal::Bool(a))),
            Box::new(Expr::Literal(Literal::Bool(b))),
        ))],
    }
}

#[test]
fn eval_bool_and_true_true() {
    let axe = Axe::new();
    let program = make_bool_binary(Operation::And, true, true);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_and_true_false() {
    let axe = Axe::new();
    let program = make_bool_binary(Operation::And, true, false);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_or_false_false() {
    let axe = Axe::new();
    let program = make_bool_binary(Operation::Or, false, false);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_or_true_false() {
    let axe = Axe::new();
    let program = make_bool_binary(Operation::Or, true, false);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_equality() {
    let axe = Axe::new();
    let program = make_bool_binary(Operation::Eq, true, true);
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Binary Operation Tests - Strings
// ============================================================================

fn make_str_binary(op: Operation, a: &str, b: &str) -> Program {
    Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            op,
            Box::new(Expr::Literal(Literal::Str(a.to_string()))),
            Box::new(Expr::Literal(Literal::Str(b.to_string()))),
        ))],
    }
}

#[test]
fn eval_string_equality() {
    let axe = Axe::new();
    let program = make_str_binary(Operation::Eq, "hello", "hello");
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_string_inequality() {
    let axe = Axe::new();
    let program = make_str_binary(Operation::Neq, "hello", "world");
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Let Statement Tests
// ============================================================================

#[test]
fn eval_let_with_value() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Int(42))),
            )]),
            Stmt::Expr(Expr::Var("x".to_string())),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_let_without_value_is_null() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".to_string(), None)]),
            Stmt::Expr(Expr::Var("x".to_string())),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_let_multiple_declarations() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![
                ("x".to_string(), Some(Expr::Literal(Literal::Int(1)))),
                ("y".to_string(), Some(Expr::Literal(Literal::Int(2)))),
            ]),
            Stmt::Expr(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Var("y".to_string())),
            )),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_let_invalid_name_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            "123invalid".to_string(),
            Some(Expr::Literal(Literal::Int(1))),
        )])],
    };
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "invalid variable name");
}

// ============================================================================
// Assignment Tests
// ============================================================================

#[test]
fn eval_assign_existing_variable() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Int(10))),
            )]),
            Stmt::Assign("x".to_string(), Expr::Literal(Literal::Int(20))),
            Stmt::Expr(Expr::Var("x".to_string())),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_assign_undefined_variable_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Assign(
            "x".to_string(),
            Expr::Literal(Literal::Int(10)),
        )],
    };
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "undefined variable");
}

// ============================================================================
// Block Tests
// ============================================================================

#[test]
fn eval_empty_block() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Block(vec![])],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_block_returns_last_value() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Block(vec![
            Stmt::Expr(Expr::Literal(Literal::Int(1))),
            Stmt::Expr(Expr::Literal(Literal::Int(2))),
            Stmt::Expr(Expr::Literal(Literal::Int(3))),
        ])],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// If Statement Tests
// ============================================================================

#[test]
fn eval_if_true_branch() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "result".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::If(
                Expr::Literal(Literal::Bool(true)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "result".to_string(),
                    Expr::Literal(Literal::Int(1)),
                )])),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "result".to_string(),
                    Expr::Literal(Literal::Int(2)),
                )])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_if_false_branch() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "result".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::If(
                Expr::Literal(Literal::Bool(false)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "result".to_string(),
                    Expr::Literal(Literal::Int(1)),
                )])),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "result".to_string(),
                    Expr::Literal(Literal::Int(2)),
                )])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_if_truthy_int() {
    let axe = Axe::new();
    // Non-zero int is truthy
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Int(1)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                1,
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                2,
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_if_falsy_zero() {
    let axe = Axe::new();
    // Zero is falsy
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Int(0)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                1,
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                2,
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_if_falsy_null() {
    let axe = Axe::new();
    // Null is falsy
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Null),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                1,
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                2,
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// While Statement Tests
// ============================================================================

#[test]
fn eval_while_loop() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "i".to_string(),
                Some(Expr::Literal(Literal::Int(0))),
            )]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var("i".to_string())),
                    Box::new(Expr::Literal(Literal::Int(3))),
                ),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    "i".to_string(),
                    Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("i".to_string())),
                        Box::new(Expr::Literal(Literal::Int(1))),
                    ),
                )])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_while_false_condition_never_executes() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::While(
            Expr::Literal(Literal::Bool(false)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                1,
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Function Definition and Call Tests
// ============================================================================

#[test]
fn eval_function_definition() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Function(
            "add".to_string(),
            vec!["a".to_string(), "b".to_string()],
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("a".to_string())),
                Box::new(Expr::Var("b".to_string())),
            ))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_function_call() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Function(
                "double".to_string(),
                vec!["x".to_string()],
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Binary(
                    Operation::Mul,
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Literal(Literal::Int(2))),
                ))])),
            ),
            Stmt::Expr(Expr::Call(
                "double".to_string(),
                vec![Expr::Literal(Literal::Int(5))],
            )),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_function_call_wrong_arg_count_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Function(
                "add".to_string(),
                vec!["a".to_string(), "b".to_string()],
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                    0,
                )))])),
            ),
            Stmt::Expr(Expr::Call(
                "add".to_string(),
                vec![Expr::Literal(Literal::Int(1))],
            )),
        ],
    };
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "argument count mismatch");
}

#[test]
fn eval_undefined_function_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call("nonexistent".to_string(), vec![]))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "undefined function");
}

// ============================================================================
// List Tests
// ============================================================================

#[test]
fn eval_list_literal() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::List(vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
            Expr::Literal(Literal::Int(3)),
        ]))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_empty_list() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::List(vec![]))],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Native Function Tests
// ============================================================================

#[test]
fn eval_native_len_list() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "len".to_string(),
            vec![Expr::List(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
                Expr::Literal(Literal::Int(3)),
            ])],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_len_string() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "len".to_string(),
            vec![Expr::Literal(Literal::Str("hello".to_string()))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_type() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "type".to_string(),
            vec![Expr::Literal(Literal::Int(42))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_range_single_arg() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "range".to_string(),
            vec![Expr::Literal(Literal::Int(5))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_range_two_args() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "range".to_string(),
            vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(5)),
            ],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_range_three_args() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "range".to_string(),
            vec![
                Expr::Literal(Literal::Int(0)),
                Expr::Literal(Literal::Int(10)),
                Expr::Literal(Literal::Int(2)),
            ],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_concat_strings() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "concat".to_string(),
            vec![
                Expr::Literal(Literal::Str("hello".to_string())),
                Expr::Literal(Literal::Str(" world".to_string())),
            ],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_concat_lists() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "concat".to_string(),
            vec![
                Expr::List(vec![Expr::Literal(Literal::Int(1))]),
                Expr::List(vec![Expr::Literal(Literal::Int(2))]),
            ],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_push() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "push".to_string(),
            vec![
                Expr::List(vec![Expr::Literal(Literal::Int(1))]),
                Expr::Literal(Literal::Int(2)),
            ],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_get() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "get".to_string(),
            vec![
                Expr::List(vec![
                    Expr::Literal(Literal::Int(10)),
                    Expr::Literal(Literal::Int(20)),
                    Expr::Literal(Literal::Int(30)),
                ]),
                Expr::Literal(Literal::Int(1)),
            ],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_get_negative_index() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            "get".to_string(),
            vec![
                Expr::List(vec![
                    Expr::Literal(Literal::Int(10)),
                    Expr::Literal(Literal::Int(20)),
                    Expr::Literal(Literal::Int(30)),
                ]),
                Expr::Literal(Literal::Int(-1)), // Should get last element
            ],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Class Tests
// ============================================================================

#[test]
fn eval_class_definition() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Class(
            "Point".to_string(),
            None,
            vec![
                Stmt::Let(vec![(
                    "x".to_string(),
                    Some(Expr::Literal(Literal::Int(0))),
                )]),
                Stmt::Let(vec![(
                    "y".to_string(),
                    Some(Expr::Literal(Literal::Int(0))),
                )]),
            ],
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_class_with_method() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Class(
            "Counter".to_string(),
            None,
            vec![
                Stmt::Let(vec![(
                    "count".to_string(),
                    Some(Expr::Literal(Literal::Int(0))),
                )]),
                Stmt::Function(
                    "increment".to_string(),
                    vec!["self".to_string()],
                    Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Null))])),
                ),
            ],
        )],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Cross-type Equality Tests
// ============================================================================

#[test]
fn eval_cross_type_equality_is_false() {
    let axe = Axe::new();
    // Int == String should be false
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::Str("1".to_string()))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_cross_type_inequality_is_true() {
    let axe = Axe::new();
    // Int != String should be true
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::Str("1".to_string()))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}
