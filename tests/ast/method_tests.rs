use axe::ast::{Expr, Literal, Program, Stmt};
use axe::{Axe, Context, Value};

// ============================================================================
// String Method Tests
// ============================================================================

#[test]
fn string_len_method() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            context.intern("len"),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    // Should return 5
    assert_eq!(format!("{}", result.unwrap()), "5");
}

#[test]
fn string_len_empty() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("")))),
            context.intern("len"),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "0");
}

#[test]
fn string_concat_method() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            context.intern("concat"),
            vec![Expr::Literal(Literal::Str(context.intern(" world")))],
        ))],
    };
    let result = axe.run(program).unwrap();
    match result {
        Value::Literal(Literal::Str(s)) => assert_eq!(context.resolve(s), "hello world"),
        _ => panic!("Expected string, got {:?}", result),
    }
}

#[test]
fn string_concat_multiple() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("a")))),
            context.intern("concat"),
            vec![
                Expr::Literal(Literal::Str(context.intern("b"))),
                Expr::Literal(Literal::Str(context.intern("c"))),
            ],
        ))],
    };
    let result = axe.run(program).unwrap();
    match result {
        Value::Literal(Literal::Str(s)) => assert_eq!(context.resolve(s), "abc"),
        _ => panic!("Expected string, got {:?}", result),
    }
}

#[test]
fn string_unknown_method_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            context.intern("unknown"),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

// ============================================================================
// List Method Tests
// ============================================================================

#[test]
fn list_len_method() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "3");
}

#[test]
fn list_len_empty() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![])),
            context.intern("len"),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "0");
}

#[test]
fn list_push_method() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            context.intern("push"),
            vec![Expr::Literal(Literal::Int(2))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "[1, 2]");
}

#[test]
fn list_get_method() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "20");
}

#[test]
fn list_get_negative_index() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![
                Expr::Literal(Literal::Int(10)),
                Expr::Literal(Literal::Int(20)),
                Expr::Literal(Literal::Int(30)),
            ])),
            context.intern("get"),
            vec![Expr::Literal(Literal::Int(-1))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "30");
}

#[test]
fn list_get_out_of_bounds_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            context.intern("get"),
            vec![Expr::Literal(Literal::Int(5))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

#[test]
fn list_concat_method() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            context.intern("concat"),
            vec![Expr::List(vec![Expr::Literal(Literal::Int(2))])],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "[1, 2]");
}

#[test]
fn list_concat_multiple() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            context.intern("concat"),
            vec![
                Expr::List(vec![Expr::Literal(Literal::Int(2))]),
                Expr::List(vec![Expr::Literal(Literal::Int(3))]),
            ],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "[1, 2, 3]");
}

#[test]
fn list_unknown_method_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![])),
            context.intern("unknown"),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

// ============================================================================
// Chained Method Calls
// ============================================================================

#[test]
fn list_chained_push() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // [1].push(2).push(3)
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::MethodCall(
                Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
                context.intern("push"),
                vec![Expr::Literal(Literal::Int(2))],
            )),
            context.intern("push"),
            vec![Expr::Literal(Literal::Int(3))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "[1, 2, 3]");
}

#[test]
fn list_push_then_len() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // [1].push(2).len()
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::MethodCall(
                Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
                context.intern("push"),
                vec![Expr::Literal(Literal::Int(2))],
            )),
            context.intern("len"),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "2");
}

// ============================================================================
// Method Call on Variable
// ============================================================================

#[test]
fn method_call_on_variable() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    // let x = "hello"; x.len()
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("x"),
                Some(Expr::Literal(Literal::Str(context.intern("hello")))),
                None,
            )]),
            Stmt::Expr(Expr::MethodCall(
                Box::new(Expr::Var(context.intern("x"))),
                context.intern("len"),
                vec![],
            )),
        ],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "5");
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn method_call_on_int_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Int(42))),
            context.intern("len"),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

#[test]
fn string_len_with_args_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str(context.intern("hello")))),
            context.intern("len"),
            vec![Expr::Literal(Literal::Int(1))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

#[test]
fn list_push_wrong_args_fails() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![])),
            context.intern("push"),
            vec![], // push requires 1 argument
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}
