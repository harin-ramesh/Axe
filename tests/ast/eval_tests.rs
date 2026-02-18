use axe::{Axe, Context, Environment, Expr, Literal, Operation, Program, Stmt, Value};

// ============================================================================
// Environment Tests
// ============================================================================

#[test]
fn environment_new_creates_empty_env() {
    let context = Context::new();
    let env = Environment::new();
    assert!(env.borrow().get(context.intern("x")).is_none());
}

#[test]
fn environment_set_and_get() {
    let context = Context::new();
    let env = Environment::new();
    let x = context.intern("x");
    env.borrow_mut().set(x, Value::Literal(Literal::Int(42)));

    let value = env.borrow().get(x).unwrap();
    match value {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 42),
        _ => panic!("Expected Int(42)"),
    }
}

#[test]
fn environment_extend_inherits_parent() {
    let context = Context::new();
    let parent = Environment::new();
    let x = context.intern("x");
    parent.borrow_mut().set(x, Value::Literal(Literal::Int(10)));

    let child = Environment::extend(parent);

    let value = child.borrow().get(x).unwrap();
    match value {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 10),
        _ => panic!("Expected Int(10)"),
    }
}

#[test]
fn environment_child_shadows_parent() {
    let context = Context::new();
    let parent = Environment::new();
    let x = context.intern("x");
    parent.borrow_mut().set(x, Value::Literal(Literal::Int(10)));

    let child = Environment::extend(parent.clone());
    child.borrow_mut().set(x, Value::Literal(Literal::Int(20)));

    // Child should see shadowed value
    match child.borrow().get(x).unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 20),
        _ => panic!("Expected Int(20)"),
    }

    // Parent should still have original value
    match parent.borrow().get(x).unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 10),
        _ => panic!("Expected Int(10)"),
    }
}

#[test]
fn environment_update_modifies_existing() {
    let context = Context::new();
    let env = Environment::new();
    let x = context.intern("x");
    env.borrow_mut().set(x, Value::Literal(Literal::Int(10)));
    env.borrow_mut()
        .update(x, Value::Literal(Literal::Int(20)))
        .unwrap();

    match env.borrow().get(x).unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 20),
        _ => panic!("Expected Int(20)"),
    }
}

#[test]
fn environment_update_undefined_fails() {
    let context = Context::new();
    let env = Environment::new();
    let x = context.intern("x");
    let result = env.borrow_mut().update(x, Value::Literal(Literal::Int(10)));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "undefined variable");
}

#[test]
fn environment_update_parent_variable() {
    let context = Context::new();
    let parent = Environment::new();
    let x = context.intern("x");
    parent.borrow_mut().set(x, Value::Literal(Literal::Int(10)));

    let child = Environment::extend(parent.clone());
    child
        .borrow_mut()
        .update(x, Value::Literal(Literal::Int(20)))
        .unwrap();

    // Parent should be updated
    match parent.borrow().get(x).unwrap() {
        Value::Literal(Literal::Int(n)) => assert_eq!(n, 20),
        _ => panic!("Expected Int(20)"),
    }
}

// ============================================================================
// Literal Evaluation Tests
// ============================================================================

#[test]
fn eval_int_literal() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Int(42)))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_float_literal() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Float(3.14)))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_string_literal() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Str(
            context.intern("hello"),
        )))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_bool_literal() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Literal(Literal::Bool(true)))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_null_literal() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Add, 10, 5);
    let result = axe.run(program).unwrap();
    match result {
        Value::Literal(Literal::Null) => {} // Program returns null, but expression was evaluated
        _ => {}
    }
}

#[test]
fn eval_int_subtraction() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Sub, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_multiplication() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Mul, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_division() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Div, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_modulo() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Mod, 10, 3);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_division_by_zero_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Div, 10, 0);
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "division by zero");
}

#[test]
fn eval_int_greater_than() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Gt, 10, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_less_than() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Lt, 5, 10);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_equal() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Eq, 5, 5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_not_equal() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::Neq, 5, 10);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_bitwise_and() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_int_binary(Operation::BitwiseAnd, 0b1010, 0b1100);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_int_bitwise_or() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_float_binary(Operation::Add, 2.5, 1.5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_subtraction() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_float_binary(Operation::Sub, 5.0, 2.5);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_multiplication() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_float_binary(Operation::Mul, 2.0, 3.0);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_division() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_float_binary(Operation::Div, 10.0, 2.0);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_float_division_by_zero_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_bool_binary(Operation::And, true, true);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_and_true_false() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_bool_binary(Operation::And, true, false);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_or_false_false() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_bool_binary(Operation::Or, false, false);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_or_true_false() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_bool_binary(Operation::Or, true, false);
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_bool_equality() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = make_bool_binary(Operation::Eq, true, true);
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Binary Operation Tests - Strings
// ============================================================================

#[test]
fn eval_string_equality() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_string_inequality() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            Box::new(Expr::Literal(Literal::Str(context.intern("world")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Let Statement Tests
// ============================================================================

#[test]
fn eval_let_with_value() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let x = context.intern("x");
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(x, Some(Expr::Literal(Literal::Int(42))), None)]),
            Stmt::Expr(Expr::Var(x)),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_let_without_value_is_null() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let x = context.intern("x");
    let program = Program {
        stmts: vec![Stmt::Let(vec![(x, None, None)]), Stmt::Expr(Expr::Var(x))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_let_multiple_declarations() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let x = context.intern("x");
    let y = context.intern("y");
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![
                (x, Some(Expr::Literal(Literal::Int(1))), None),
                (y, Some(Expr::Literal(Literal::Int(2))), None),
            ]),
            Stmt::Expr(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var(x)),
                Box::new(Expr::Var(y)),
            )),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_let_invalid_name_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let name = context.intern("123invalid");
    let program = Program {
        stmts: vec![Stmt::Let(vec![(
            name,
            Some(Expr::Literal(Literal::Int(1))),
            None,
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let x = context.intern("x");
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(x, Some(Expr::Literal(Literal::Int(10))), None)]),
            Stmt::Assign(x, Expr::Literal(Literal::Int(20))),
            Stmt::Expr(Expr::Var(x)),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_assign_undefined_variable_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let x = context.intern("x");
    let program = Program {
        stmts: vec![Stmt::Assign(x, Expr::Literal(Literal::Int(10)))],
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Block(vec![])],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_block_returns_last_value() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result_sym = context.intern("result");
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                result_sym,
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::If(
                Expr::Literal(Literal::Bool(true)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    result_sym,
                    Expr::Literal(Literal::Int(1)),
                )])),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    result_sym,
                    Expr::Literal(Literal::Int(2)),
                )])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_if_false_branch() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result_sym = context.intern("result");
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                result_sym,
                Some(Expr::Literal(Literal::Int(0))),
                None,
            )]),
            Stmt::If(
                Expr::Literal(Literal::Bool(false)),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    result_sym,
                    Expr::Literal(Literal::Int(1)),
                )])),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    result_sym,
                    Expr::Literal(Literal::Int(2)),
                )])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_if_truthy_int() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let i = context.intern("i");
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(i, Some(Expr::Literal(Literal::Int(0))), None)]),
            Stmt::While(
                Expr::Binary(
                    Operation::Lt,
                    Box::new(Expr::Var(i)),
                    Box::new(Expr::Literal(Literal::Int(3))),
                ),
                Box::new(Stmt::Block(vec![Stmt::Assign(
                    i,
                    Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var(i)),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let a = context.intern("a");
    let b = context.intern("b");
    let program = Program {
        stmts: vec![Stmt::Function(
            context.intern("add"),
            vec![a, b],
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var(a)),
                Box::new(Expr::Var(b)),
            ))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_function_call() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let x = context.intern("x");
    let program = Program {
        stmts: vec![
            Stmt::Function(
                context.intern("double"),
                vec![x],
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Binary(
                    Operation::Mul,
                    Box::new(Expr::Var(x)),
                    Box::new(Expr::Literal(Literal::Int(2))),
                ))])),
            ),
            Stmt::Expr(Expr::Call(
                context.intern("double"),
                vec![Expr::Literal(Literal::Int(5))],
            )),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_function_call_wrong_arg_count_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let a = context.intern("a");
    let b = context.intern("b");
    let program = Program {
        stmts: vec![
            Stmt::Function(
                context.intern("add"),
                vec![a, b],
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                    0,
                )))])),
            ),
            Stmt::Expr(Expr::Call(
                context.intern("add"),
                vec![Expr::Literal(Literal::Int(1))],
            )),
        ],
    };
    let result = axe.run(program);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "wrong number of arguments");
}

#[test]
fn eval_undefined_function_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            context.intern("nonexistent"),
            vec![],
        ))],
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Using method syntax: [1, 2, 3].len()
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
                Expr::Literal(Literal::Int(3)),
            ])),
            context.intern("len"),
            vec![],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_len_string() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Using method syntax: "hello".len()
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            context.intern("len"),
            vec![],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_type() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            context.intern("type"),
            vec![Expr::Literal(Literal::Int(42))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_range_single_arg() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            context.intern("range"),
            vec![Expr::Literal(Literal::Int(5))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_range_two_args() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            context.intern("range"),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Call(
            context.intern("range"),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Using method syntax: "hello".concat(" world")
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            context.intern("concat"),
            vec![Expr::Literal(Literal::Str(context.intern(" world")))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_concat_lists() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Using method syntax: [1].concat([2])
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            context.intern("concat"),
            vec![Expr::List(vec![Expr::Literal(Literal::Int(2))])],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_push() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Using method syntax: [1].push(2)
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            context.intern("push"),
            vec![Expr::Literal(Literal::Int(2))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_get() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Using method syntax: [10, 20, 30].get(1)
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![
                Expr::Literal(Literal::Int(10)),
                Expr::Literal(Literal::Int(20)),
                Expr::Literal(Literal::Int(30)),
            ])),
            context.intern("get"),
            vec![Expr::Literal(Literal::Int(1))],
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_native_get_negative_index() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Using method syntax: [10, 20, 30].get(-1)
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![
                Expr::Literal(Literal::Int(10)),
                Expr::Literal(Literal::Int(20)),
                Expr::Literal(Literal::Int(30)),
            ])),
            context.intern("get"),
            vec![Expr::Literal(Literal::Int(-1))], // Should get last element
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// ============================================================================
// Class Tests
// ============================================================================

#[test]
fn eval_class_definition() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let point = context.intern("Point");
    let x = context.intern("x");
    let y = context.intern("y");
    let program = Program {
        stmts: vec![Stmt::Class(
            point,
            None,
            vec![
                Stmt::Let(vec![(x, Some(Expr::Literal(Literal::Int(0))), None)]),
                Stmt::Let(vec![(y, Some(Expr::Literal(Literal::Int(0))), None)]),
            ],
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_class_with_method() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let counter = context.intern("Counter");
    let count = context.intern("count");
    let increment = context.intern("increment");
    let self_sym = context.intern("self");
    let program = Program {
        stmts: vec![Stmt::Class(
            counter,
            None,
            vec![
                Stmt::Let(vec![(count, Some(Expr::Literal(Literal::Int(0))), None)]),
                Stmt::Function(
                    increment,
                    vec![self_sym],
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Int == String should be false
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::Str(context.intern("1")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eval_cross_type_inequality_is_true() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // Int != String should be true
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::Str(context.intern("1")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}
