use axe::{Axe, Expr, Literal, Operation, Program, Stmt};

// Greater than tests
#[test]
fn gt_int_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn gt_int_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn gt_float_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Literal(Literal::Float(10.5))),
            Box::new(Expr::Literal(Literal::Float(5.5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Less than tests
#[test]
fn lt_int_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Lt,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn lt_int_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Lt,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Greater than or equal tests
#[test]
fn gte_int_true_greater() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gte,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn gte_int_true_equal() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gte,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn gte_int_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gte,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Less than or equal tests
#[test]
fn lte_int_true_less() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Lte,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn lte_int_true_equal() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Lte,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn lte_int_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Lte,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Equality tests
#[test]
fn eq_int_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_int_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_float_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Float(10.5))),
            Box::new(Expr::Literal(Literal::Float(10.5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_string_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_string_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            Box::new(Expr::Literal(Literal::Str("world".to_string()))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_bool_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Bool(true))),
            Box::new(Expr::Literal(Literal::Bool(true))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_null_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Null)),
            Box::new(Expr::Literal(Literal::Null)),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Not equal tests
#[test]
fn neq_int_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(5))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn neq_int_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(10))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Cross-type comparisons
#[test]
fn eq_cross_type_false() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Str("10".to_string()))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn neq_cross_type_true() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Str("10".to_string()))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Comparison with variables
#[test]
fn comparison_with_variables() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                "x".to_string(),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Let(vec![(
                "y".to_string(),
                Some(Expr::Literal(Literal::Int(5))),
                None,
            )]),
            Stmt::Expr(Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Var("x".to_string())),
                Box::new(Expr::Var("y".to_string())),
            )),
        ],
    };
    assert!(axe.run(program).is_ok());
}

// Comparison in if expressions
#[test]
fn comparison_in_if_condition() {
    let mut axe = Axe::new();
    // if (10 > 5) { "yes" } else { "no" }
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Literal(Literal::Int(10))),
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
fn comparison_false_in_if_condition() {
    let mut axe = Axe::new();
    // if (5 > 10) { "yes" } else { "no" }
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(10))),
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

// Boolean values in if expressions
#[test]
fn bool_true_in_if_condition() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Bool(true)),
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
fn bool_false_in_if_condition() {
    let mut axe = Axe::new();
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Bool(false)),
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

// Nested comparisons
#[test]
fn nested_comparison_expressions() {
    let mut axe = Axe::new();
    // (10 > 5) == (3 < 7)  -> true == true -> true
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Literal(Literal::Int(10))),
                Box::new(Expr::Literal(Literal::Int(5))),
            )),
            Box::new(Expr::Binary(
                Operation::Lt,
                Box::new(Expr::Literal(Literal::Int(3))),
                Box::new(Expr::Literal(Literal::Int(7))),
            )),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn comparison_type_error() {
    let mut axe = Axe::new();
    // Can't use > on strings
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            Box::new(Expr::Literal(Literal::Str("world".to_string()))),
        ))],
    };
    // This should fail with type error
    let result = axe.run(program);
    assert!(result.is_err());
}
