use eva::{Eva, Expr};

#[test]
fn valid_variable_names() {
    let eva = Eva::new();
    
    // Valid names starting with letter
    eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();
    eva.eval(Expr::Set("myVar".into(), Box::new(Expr::Int(2)))).unwrap();
    eva.eval(Expr::Set("var123".into(), Box::new(Expr::Int(3)))).unwrap();
    
    // Valid names starting with underscore
    eva.eval(Expr::Set("_private".into(), Box::new(Expr::Int(4)))).unwrap();
    eva.eval(Expr::Set("_".into(), Box::new(Expr::Int(5)))).unwrap();
    eva.eval(Expr::Set("_123".into(), Box::new(Expr::Int(6)))).unwrap();
    
    // Valid names with underscores
    eva.eval(Expr::Set("my_var".into(), Box::new(Expr::Int(7)))).unwrap();
    eva.eval(Expr::Set("CONSTANT_VALUE".into(), Box::new(Expr::Int(8)))).unwrap();
}

#[test]
fn invalid_variable_name_starting_with_number() {
    let eva = Eva::new();
    let err = eva.eval(Expr::Set("123var".into(), Box::new(Expr::Int(1)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_with_special_chars() {
    let eva = Eva::new();
    
    let err = eva.eval(Expr::Set("my-var".into(), Box::new(Expr::Int(1)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
    
    let err = eva.eval(Expr::Set("my.var".into(), Box::new(Expr::Int(1)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
    
    let err = eva.eval(Expr::Set("my var".into(), Box::new(Expr::Int(1)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
    
    let err = eva.eval(Expr::Set("my@var".into(), Box::new(Expr::Int(1)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_empty() {
    let eva = Eva::new();
    let err = eva.eval(Expr::Set("".into(), Box::new(Expr::Int(1)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn invalid_variable_name_on_get() {
    let eva = Eva::new();
    
    // Test that invalid names also fail on Var access
    let err = eva.eval(Expr::Var("123var".into())).unwrap_err();
    assert_eq!(err, "invalid variable name");
    
    let err = eva.eval(Expr::Var("my-var".into())).unwrap_err();
    assert_eq!(err, "invalid variable name");
}
