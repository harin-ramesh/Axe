use axe::{Axe, Expr, Operation, Value};

#[test]
fn assign_to_existing_global_variable() {
    let axe = Axe::new();
    
    // Create variable x = 10
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    
    // Reassign x = 20
    let result = axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(20)))).unwrap();
    assert_eq!(result, Value::Int(20));
    
    // Verify x is now 20
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(20));
}

#[test]
fn let_creates_variable_if_not_exists() {
    let axe = Axe::new();
    
    // let creates a variable even if it doesn't exist
    let result = axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    assert_eq!(result, Value::Int(10));
    
    // Verify it was created
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(10));
}

#[test]
fn assign_with_invalid_name_fails() {
    let axe = Axe::new();
    
    // Try to assign with invalid variable name
    let err = axe.eval(Expr::Set("123invalid".into(), Box::new(Expr::Int(10)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn assign_using_expression() {
    let axe = Axe::new();
    
    // Create variable x = 5
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(5)))).unwrap();
    
    // Reassign x = x * 2
    let result = axe.eval(Expr::Set(
        "x".into(),
        Box::new(Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Var("x".into())),
            Box::new(Expr::Int(2)),
        )),
    )).unwrap();
    assert_eq!(result, Value::Int(10));
    
    // Verify x is now 10
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(10));
}

#[test]
fn let_in_block_updates_variable() {
    let axe = Axe::new();
    
    // Create global variable x = 10
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    
    // Block updates x (blocks don't create new scope)
    let block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(100))),
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(block).unwrap();
    assert_eq!(result, Value::Int(100));
    
    // Global x should now be 100 (block updated it in same scope)
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(100));
}

#[test]
fn let_in_nested_block_updates() {
    let axe = Axe::new();
    
    // Outer block: create x = 10
    let block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        
        // Inner block: updates x = 20 (same scope)
        Expr::Block(vec![
            Expr::Set("x".into(), Box::new(Expr::Int(20))),
        ]),
        
        // x should now be 20 (inner block updated it)
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(block).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn let_in_same_scope_updates() {
    let axe = Axe::new();
    
    // Global x = 1
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();
    
    // Update in same scope
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(100)))).unwrap();
    
    // Global x should now be 100 (updated in same scope)
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(100));
}

#[test]
fn let_updates_through_blocks() {
    let axe = Axe::new();
    
    // Global x = 1
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();
    
    let outer_block = Expr::Block(vec![
        // Outer block updates global x to 10
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        
        // Inner block updates x to 20 (same scope)
        Expr::Block(vec![
            Expr::Set("x".into(), Box::new(Expr::Int(20))),
        ]),
        
        // x should now be 20
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(outer_block).unwrap();
    assert_eq!(result, Value::Int(20));
    
    // Global x should also be 20 (blocks updated it)
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(20));
}
