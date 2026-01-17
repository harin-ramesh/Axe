use axe::{Axe, Expr};

#[test]
fn valid_variable_names() {
    let axe = Axe::new();

    // Valid names starting with letter
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(1))))
        .unwrap();
    axe.eval(Expr::Set("myVar".into(), Box::new(Expr::Int(2))))
        .unwrap();
    axe.eval(Expr::Set("var123".into(), Box::new(Expr::Int(3))))
        .unwrap();

    // Valid names starting with underscore
    axe.eval(Expr::Set("_private".into(), Box::new(Expr::Int(4))))
        .unwrap();
    axe.eval(Expr::Set("_".into(), Box::new(Expr::Int(5))))
        .unwrap();
    axe.eval(Expr::Set("_123".into(), Box::new(Expr::Int(6))))
        .unwrap();

    // Valid names with underscores
    axe.eval(Expr::Set("my_var".into(), Box::new(Expr::Int(7))))
        .unwrap();
    axe.eval(Expr::Set("CONSTANT_VALUE".into(), Box::new(Expr::Int(8))))
        .unwrap();
}

#[test]
fn invalid_variable_name_starting_with_number() {
    let axe = Axe::new();
    let err = axe
        .eval(Expr::Set("123var".into(), Box::new(Expr::Int(1))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_with_special_chars() {
    let axe = Axe::new();

    let err = axe
        .eval(Expr::Set("my-var".into(), Box::new(Expr::Int(1))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");

    let err = axe
        .eval(Expr::Set("my.var".into(), Box::new(Expr::Int(1))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");

    let err = axe
        .eval(Expr::Set("my var".into(), Box::new(Expr::Int(1))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");

    let err = axe
        .eval(Expr::Set("my@var".into(), Box::new(Expr::Int(1))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_empty() {
    let axe = Axe::new();
    let err = axe
        .eval(Expr::Set("".into(), Box::new(Expr::Int(1))))
        .unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_on_get() {
    let axe = Axe::new();

    // Test that invalid names also fail on Var access
    let err = axe.eval(Expr::Var("123var".into())).unwrap_err();
    assert_eq!(err, "invalid variable name");

    let err = axe.eval(Expr::Var("my-var".into())).unwrap_err();
    assert_eq!(err, "invalid variable name");
}
