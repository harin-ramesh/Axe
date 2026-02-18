use axe::{Axe, Context, Expr, Literal, Operation, Program, Stmt};

#[test]
fn if_with_truthy_condition() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Add,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("yes"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("no"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_block_branches() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(context.intern("x"), None, None)]),
            Stmt::If(
                Expr::Literal(Literal::Int(1)),
                Box::new(Stmt::Block(vec![
                    Stmt::Assign(context.intern("x"), Expr::Literal(Literal::Int(10))),
                    Stmt::Expr(Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var(context.intern("x"))),
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("x"),
                Some(Expr::Literal(Literal::Int(5))),
                None,
            )]),
            Stmt::If(
                Expr::Var(context.intern("x")),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    context.intern("truthy"),
                )))])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    context.intern("falsy"),
                )))])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_non_zero_numbers_are_truthy() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Bool(true)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("then"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("else"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_bool_false_is_falsy() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Literal(Literal::Bool(false)),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("then"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("else"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_comparison_as_condition() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Literal(Literal::Int(10))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("greater"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("not greater"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_equality_check() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Eq,
                Box::new(Expr::Literal(Literal::Int(10))),
                Box::new(Expr::Literal(Literal::Int(10))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("equal"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("not equal"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_bool_variable() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("x"),
                Some(Expr::Literal(Literal::Bool(true))),
                None,
            )]),
            Stmt::If(
                Expr::Var(context.intern("x")),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    context.intern("yes"),
                )))])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    context.intern("no"),
                )))])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_with_false_bool_variable() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                context.intern("x"),
                Some(Expr::Literal(Literal::Bool(false))),
                None,
            )]),
            Stmt::If(
                Expr::Var(context.intern("x")),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    context.intern("yes"),
                )))])),
                Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    context.intern("no"),
                )))])),
            ),
        ],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn if_falsy_values() {
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("truthy"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                context.intern("falsy"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}
