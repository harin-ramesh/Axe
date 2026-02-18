use axe::{Axe, Context};
use axe::{Expr, Literal, Operation, Program, Stmt};

// Greater than tests
#[test]
fn gt_int_true() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Str(ctx.intern("hello")))),
            Box::new(Expr::Literal(Literal::Str(ctx.intern("hello")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_string_false() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Str(ctx.intern("hello")))),
            Box::new(Expr::Literal(Literal::Str(ctx.intern("world")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn eq_bool_true() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Str(ctx.intern("10")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn neq_cross_type_true() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Str(ctx.intern("10")))),
        ))],
    };
    assert!(axe.run(program).is_ok());
}

// Comparison with variables
#[test]
fn comparison_with_variables() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    let program = Program {
        stmts: vec![
            Stmt::Let(vec![(
                ctx.intern("x"),
                Some(Expr::Literal(Literal::Int(10))),
                None,
            )]),
            Stmt::Let(vec![(
                ctx.intern("y"),
                Some(Expr::Literal(Literal::Int(5))),
                None,
            )]),
            Stmt::Expr(Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Var(ctx.intern("x"))),
                Box::new(Expr::Var(ctx.intern("y"))),
            )),
        ],
    };
    assert!(axe.run(program).is_ok());
}

// Comparison in if expressions
#[test]
fn comparison_in_if_condition() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    // if (10 > 5) { "yes" } else { "no" }
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Literal(Literal::Int(10))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                ctx.intern("yes"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                ctx.intern("no"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

#[test]
fn comparison_false_in_if_condition() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    // if (5 > 10) { "yes" } else { "no" }
    let program = Program {
        stmts: vec![Stmt::If(
            Expr::Binary(
                Operation::Gt,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(10))),
            ),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                ctx.intern("yes"),
            )))])),
            Box::new(Stmt::Block(vec![Stmt::Expr(Expr::Literal(Literal::Str(
                ctx.intern("no"),
            )))])),
        )],
    };
    assert!(axe.run(program).is_ok());
}

// Boolean values in if expressions
#[test]
fn bool_true_in_if_condition() {
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
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
    let ctx = Context::new();
    let mut axe = Axe::new(&ctx);
    // Can't use > on strings
    let program = Program {
        stmts: vec![Stmt::Expr(Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Literal(Literal::Str(ctx.intern("hello")))),
            Box::new(Expr::Literal(Literal::Str(ctx.intern("world")))),
        ))],
    };
    // This should fail with type error
    let result = axe.run(program);
    assert!(result.is_err());
}
