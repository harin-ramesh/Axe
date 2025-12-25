use axe::{Axe, Expr, Operation, Value};

#[test]
fn assign_to_existing_global_variable() {
    let axe = Axe::new();
    
    // Create variable x = 10
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    
    // Reassign x = 20
    let result = axe.eval(Expr::Assign("x".into(), Box::new(Expr::Int(20)))).unwrap();
    assert_eq!(result, Value::Int(20));
    
    // Verify x is now 20
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(20));
}

#[test]
fn assign_to_nonexistent_variable_fails() {
    let axe = Axe::new();
    
    // Try to assign to a variable that doesn't exist
    let err = axe.eval(Expr::Assign("x".into(), Box::new(Expr::Int(10)))).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn assign_with_invalid_name_fails() {
    let axe = Axe::new();
    
    // Try to assign with invalid variable name
    let err = axe.eval(Expr::Assign("123invalid".into(), Box::new(Expr::Int(10)))).unwrap_err();
    assert_eq!(err, "invalid variable name");
}

#[test]
fn assign_using_expression() {
    let axe = Axe::new();
    
    // Create variable x = 5
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(5)))).unwrap();
    
    // Reassign x = x * 2
    let result = axe.eval(Expr::Assign(
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
fn assign_in_block_to_parent_variable() {
    let axe = Axe::new();
    
    // Create global variable x = 10
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    
    // Block reassigns parent's x
    let block = Expr::Block(vec![
        Expr::Assign("x".into(), Box::new(Expr::Int(100))),
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(block).unwrap();
    assert_eq!(result, Value::Int(100));
    
    // Global x should now be 100 (modified by child block)
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(100));
}

#[test]
fn assign_in_nested_block_to_parent_variable() {
    let axe = Axe::new();
    
    // Outer block: create x = 10
    let block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        
        // Inner block: reassign parent's x = 20
        Expr::Block(vec![
            Expr::Assign("x".into(), Box::new(Expr::Int(20))),
        ]),
        
        // x should now be 20
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(block).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn set_vs_assign_shadowing() {
    let axe = Axe::new();
    
    // Global x = 1
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();
    
    // Block with Set creates new x (shadows)
    let block_with_set = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(100))),
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(block_with_set).unwrap();
    assert_eq!(result, Value::Int(100));
    
    // Global x should still be 1 (Set shadowed it)
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(1));
    
    // Block with Assign modifies parent x
    let block_with_assign = Expr::Block(vec![
        Expr::Assign("x".into(), Box::new(Expr::Int(200))),
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(block_with_assign).unwrap();
    assert_eq!(result, Value::Int(200));
    
    // Global x should now be 200 (Assign modified it)
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(200));
}

#[test]
fn assign_to_correct_scope_with_shadowing() {
    let axe = Axe::new();
    
    // Global x = 1
    axe.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();
    
    let outer_block = Expr::Block(vec![
        // Outer block creates its own x = 10 (shadows global)
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        
        // Inner block assigns to outer block's x
        Expr::Block(vec![
            Expr::Assign("x".into(), Box::new(Expr::Int(20))),
        ]),
        
        // Outer block's x should now be 20
        Expr::Var("x".into()),
    ]);
    
    let result = axe.eval(outer_block).unwrap();
    assert_eq!(result, Value::Int(20));
    
    // Global x should still be 1 (untouched)
    let x = axe.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(x, Value::Int(1));
}
