use axe::{Axe, Expr, Literal, Operation, Program, Stmt};

#[test]
fn if_with_truthy_condition() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Int(1)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                10,
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                20,
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_null_condition() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Null),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                10,
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                20,
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_expression_condition() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Add,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "yes".to_string(),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "no".to_string(),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_block_branches() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![("x".to_string(), None)]),
            Stmt::If(
                Expr::Literal(Literal::Int(1)),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign("x".to_string(), Expr::Literal(Literal::Int(10))),
                    Stmt::Expr(Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("x".to_string())),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                ])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                    0,
                )))])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn nested_if_expressions() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Int(1)),
            Box::new(Stmt::Block(vec![Stmt::If(
                Expr::Literal(Literal::Null),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                    10,
                )))])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                    20,
                )))])),
            )])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Int(
                30,
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_variable_condition() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Int(5))),
            )]),
            Stmt::If(
                Expr::Var("x".to_string()),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "truthy".to_string(),
                )))])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "falsy".to_string(),
                )))])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_non_zero_numbers_are_truthy() {
    let axe = Axe::new();
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
fn if_bool_true_is_truthy() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Bool(true)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "then".to_string(),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "else".to_string(),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_bool_false_is_falsy() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Bool(false)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "then".to_string(),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "else".to_string(),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_comparison_as_condition() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Literal(Literal::Int(10))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "greater".to_string(),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "not greater".to_string(),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_equality_check() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Eq,
                Box::new(Expr::Literal(Literal::Int(10))),
                Box::new(Expr::Literal(Literal::Int(10))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "equal".to_string(),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "not equal".to_string(),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_bool_variable() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Bool(true))),
            )]),
            Stmt::If(
                Expr::Var("x".to_string()),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "yes".to_string(),
                )))])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "no".to_string(),
                )))])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_false_bool_variable() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Bool(false))),
            )]),
            Stmt::If(
                Expr::Var("x".to_string()),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "yes".to_string(),
                )))])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "no".to_string(),
                )))])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_falsy_values() {
    let axe = Axe::new();
    // Test Null is falsy
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

#[test]
fn if_zero_is_falsy() {
    let axe = Axe::new();
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
fn if_zero_float_is_falsy() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Float(0.0)),
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
fn if_arithmetic_result_zero_is_falsy() {
    let axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "truthy".to_string(),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                "falsy".to_string(),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}
