use axe::{Axe, Condition, Expr};

#[test]
fn set_creates_new_variable() {
    let axe = Axe::new();

    // Create a variable with Set (declaration)
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10))))
        .unwrap();

    // Verify it was created
    let check = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(check.to_string(), "10");
}

#[test]
fn set_overwrites_in_same_scope() {
    let axe = Axe::new();

    // Create a variable with Set
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10))))
        .unwrap();

    // Set again in same scope overwrites
    let result = axe
        .eval(Expr::Set("x".into(), Box::new(Expr::Int(20))))
        .unwrap();
    assert_eq!(result.to_string(), "20");

    // Verify it was updated
    let check = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(check.to_string(), "20");
}

#[test]
fn assign_updates_existing_variable() {
    let axe = Axe::new();

    // Create a variable with Set
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10))))
        .unwrap();

    // Update it with Assign (reassignment)
    let result = axe
        .eval(Expr::Assign("x".into(), Box::new(Expr::Int(20))))
        .unwrap();
    assert_eq!(result.to_string(), "20");

    // Verify it was updated
    let check = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(check.to_string(), "20");
}

#[test]
fn assign_fails_on_undefined_variable() {
    let axe = Axe::new();

    // Assign to undefined variable should fail
    let err = axe
        .eval(Expr::Assign("undefined".into(), Box::new(Expr::Int(10))))
        .unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn assign_updates_parent_scope() {
    let axe = Axe::new();

    // Create global variable
    axe.eval(Expr::Set("counter".into(), Box::new(Expr::Int(0))))
        .unwrap();

    // Create function that uses Assign to update global
    let func = Expr::Function(
        "increment".into(),
        vec![],
        vec![Expr::Assign(
            "counter".into(),
            Box::new(Expr::Binary(
                axe::Operation::Add,
                Box::new(Expr::Var("counter".into())),
                Box::new(Expr::Int(1)),
            )),
        )],
    );
    axe.eval(func).unwrap();

    // Call function
    axe.eval(Expr::FunctionCall("increment".into(), vec![]))
        .unwrap();

    // Check global was updated
    let result = axe.eval(Expr::Var("counter".into())).unwrap();
    assert_eq!(result.to_string(), "1");

    // Call again
    axe.eval(Expr::FunctionCall("increment".into(), vec![]))
        .unwrap();
    let result = axe.eval(Expr::Var("counter".into())).unwrap();
    assert_eq!(result.to_string(), "2");
}

#[test]
fn set_creates_local_variable_in_function() {
    let axe = Axe::new();

    // Create global variable
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10))))
        .unwrap();

    // Function with Set creates a local variable (shadows global)
    let func = Expr::Function(
        "shadow".into(),
        vec![],
        vec![
            Expr::Set("x".into(), Box::new(Expr::Int(999))),
            Expr::Var("x".into()),
        ],
    );
    axe.eval(func).unwrap();

    // Call function - should return 999 (local)
    let result = axe
        .eval(Expr::FunctionCall("shadow".into(), vec![]))
        .unwrap();
    assert_eq!(result.to_string(), "999");

    // Global should still be 10 (Set creates local, doesn't update parent)
    let result = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(result.to_string(), "10");
}

#[test]
fn assign_in_while_loop() {
    let axe = Axe::new();

    axe.eval(Expr::Set("i".into(), Box::new(Expr::Int(0))))
        .unwrap();
    axe.eval(Expr::Set("sum".into(), Box::new(Expr::Int(0))))
        .unwrap();

    let while_loop = Expr::While(
        Condition::Binary(
            axe::Operation::Lt,
            Box::new(Condition::Var("i".into())),
            Box::new(Condition::Int(5)),
        ),
        vec![
            Expr::Assign(
                "sum".into(),
                Box::new(Expr::Binary(
                    axe::Operation::Add,
                    Box::new(Expr::Var("sum".into())),
                    Box::new(Expr::Var("i".into())),
                )),
            ),
            Expr::Assign(
                "i".into(),
                Box::new(Expr::Binary(
                    axe::Operation::Add,
                    Box::new(Expr::Var("i".into())),
                    Box::new(Expr::Int(1)),
                )),
            ),
        ],
    );

    axe.eval(while_loop).unwrap();

    let result = axe.eval(Expr::Var("sum".into())).unwrap();
    assert_eq!(result.to_string(), "10"); // 0+1+2+3+4 = 10
}

#[test]
fn set_with_invalid_name_fails() {
    let axe = Axe::new();

    let err = axe
        .eval(Expr::Set("123invalid".into(), Box::new(Expr::Int(10))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn assign_with_invalid_name_fails() {
    let axe = Axe::new();

    let err = axe
        .eval(Expr::Assign("123invalid".into(), Box::new(Expr::Int(10))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn assign_updates_through_multiple_scopes() {
    let axe = Axe::new();

    // Create global
    axe.eval(Expr::Set("value".into(), Box::new(Expr::Int(1))))
        .unwrap();

    // Outer function
    let outer_func = Expr::Function(
        "outer".into(),
        vec![],
        vec![
            // Inner function (using lambda directly since transformer doesn't recurse)
            Expr::Set(
                "inner".into(),
                Box::new(Expr::Lambda(
                    vec![],
                    vec![Expr::Assign("value".into(), Box::new(Expr::Int(100)))],
                )),
            ),
            // Call inner
            Expr::FunctionCall("inner".into(), vec![]),
        ],
    );
    axe.eval(outer_func).unwrap();

    // Call outer
    axe.eval(Expr::FunctionCall("outer".into(), vec![]))
        .unwrap();

    // Check global was updated from nested function
    let result = axe.eval(Expr::Var("value".into())).unwrap();
    assert_eq!(result.to_string(), "100");
}
