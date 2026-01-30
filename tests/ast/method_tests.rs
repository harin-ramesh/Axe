use axe::ast::{Expr, Literal, Program, Stmt};
use axe::Axe;

// ============================================================================
// String Method Tests
// ============================================================================

#[test]
fn string_len_method() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            "len".to_string(),
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
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str("".to_string()))),
            "len".to_string(),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "0");
}

#[test]
fn string_concat_method() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            "concat".to_string(),
            vec![Expr::Literal(Literal::Str(" world".to_string()))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "\"hello world\"");
}

#[test]
fn string_concat_multiple() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str("a".to_string()))),
            "concat".to_string(),
            vec![
                Expr::Literal(Literal::Str("b".to_string())),
                Expr::Literal(Literal::Str("c".to_string())),
            ],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "\"abc\"");
}

#[test]
fn string_unknown_method_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            "unknown".to_string(),
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
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
                Expr::Literal(Literal::Int(3)),
            ])),
            "len".to_string(),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "3");
}

#[test]
fn list_len_empty() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![])),
            "len".to_string(),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "0");
}

#[test]
fn list_push_method() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            "push".to_string(),
            vec![Expr::Literal(Literal::Int(2))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "[1, 2]");
}

#[test]
fn list_get_method() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![
                Expr::Literal(Literal::Int(10)),
                Expr::Literal(Literal::Int(20)),
                Expr::Literal(Literal::Int(30)),
            ])),
            "get".to_string(),
            vec![Expr::Literal(Literal::Int(1))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "20");
}

#[test]
fn list_get_negative_index() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![
                Expr::Literal(Literal::Int(10)),
                Expr::Literal(Literal::Int(20)),
                Expr::Literal(Literal::Int(30)),
            ])),
            "get".to_string(),
            vec![Expr::Literal(Literal::Int(-1))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "30");
}

#[test]
fn list_get_out_of_bounds_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            "get".to_string(),
            vec![Expr::Literal(Literal::Int(5))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

#[test]
fn list_concat_method() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            "concat".to_string(),
            vec![Expr::List(vec![Expr::Literal(Literal::Int(2))])],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "[1, 2]");
}

#[test]
fn list_concat_multiple() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
            "concat".to_string(),
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
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![])),
            "unknown".to_string(),
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
    let axe = Axe::new();
    // [1].push(2).push(3)
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::MethodCall(
                Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
                "push".to_string(),
                vec![Expr::Literal(Literal::Int(2))],
            )),
            "push".to_string(),
            vec![Expr::Literal(Literal::Int(3))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_ok());
    assert_eq!(format!("{}", result.unwrap()), "[1, 2, 3]");
}

#[test]
fn list_push_then_len() {
    let axe = Axe::new();
    // [1].push(2).len()
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::MethodCall(
                Box::new(Expr::List(vec![Expr::Literal(Literal::Int(1))])),
                "push".to_string(),
                vec![Expr::Literal(Literal::Int(2))],
            )),
            "len".to_string(),
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
    let axe = Axe::new();
    // let x = "hello"; x.len()
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Str("hello".to_string()))),
                None,
            )]),
            Stmt::Expr(Expr::MethodCall(
                Box::new(Expr::Var("x".to_string())),
                "len".to_string(),
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
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Int(42))),
            "len".to_string(),
            vec![],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

#[test]
fn string_len_with_args_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            "len".to_string(),
            vec![Expr::Literal(Literal::Int(1))],
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}

#[test]
fn list_push_wrong_args_fails() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::MethodCall(
            Box::new(Expr::List(vec![])),
            "push".to_string(),
            vec![], // push requires 1 argument
        ))],
    };
    let result = axe.run(program);
    assert!(result.is_err());
}
