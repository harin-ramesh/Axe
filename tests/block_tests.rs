use eva::{Eva, Expr, Operation, Value};

#[test]
fn block_creates_new_scope() {
    let eva = Eva::new();
    
    // Set a variable in global scope
    eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    
    // Set a variable in block scope
    let block_expr = Expr::Block(vec![
        Expr::Set("y".into(), Box::new(Expr::Int(20)))
    ]);
    
    let result = eva.eval(block_expr).unwrap();
    assert_eq!(result, Value::Int(20));
    
    // Variable y should not exist in global scope
    let err = eva.eval(Expr::Var("y".into())).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn block_can_access_parent_scope() {
    let eva = Eva::new();
    
    // Set a variable in global scope
    eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    
    // Access parent variable from block scope
    let block_expr = Expr::Block(vec![
        Expr::Var("x".into())
    ]);
    
    let result = eva.eval(block_expr).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn block_variable_shadows_parent() {
    let eva = Eva::new();
    
    // Set a variable in global scope
    eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(10)))).unwrap();
    
    // Shadow variable in block scope and use it
    let block_expr = Expr::Block(vec![
        Expr::Binary(
            Operation::Add,
            Box::new(Expr::Set("x".into(), Box::new(Expr::Int(5)))),
            Box::new(Expr::Var("x".into())),
        )
    ]);
    
    let result = eva.eval(block_expr).unwrap();
    assert_eq!(result, Value::Int(10)); // 5 + 5
    
    // Global x should still be 10
    let global_x = eva.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(global_x, Value::Int(10));
}

#[test]
fn nested_blocks() {
    let eva = Eva::new();
    
    // Set outer variable
    eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();
    
    // Nested block: outer block sets y, inner block sets z and uses both x and y
    let inner_block = Expr::Block(vec![
        Expr::Binary(
            Operation::Add,
            Box::new(Expr::Set("z".into(), Box::new(Expr::Int(3)))),
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("x".into())),
                Box::new(Expr::Var("y".into())),
            ))
        )
    ]);
    
    let outer_block = Expr::Block(vec![
        Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Set("y".into(), Box::new(Expr::Int(2)))),
            Box::new(inner_block),
        )
    ]);
    
    let result = eva.eval(outer_block).unwrap();
    // y=2, then inner: z=3, x=1, y=2, so z + (x + y) = 3 + 3 = 6, then y * 6 = 2 * 6 = 12
    assert_eq!(result, Value::Int(12));
    
    // y and z should not exist in global scope
    let err = eva.eval(Expr::Var("y".into())).unwrap_err();
    assert_eq!(err, "undefined variable");
    
    let err = eva.eval(Expr::Var("z".into())).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn block_with_multiple_expressions() {
    let eva = Eva::new();
    
    // Block with multiple expressions - returns the last one
    let block_expr = Expr::Block(vec![
        Expr::Set("a".into(), Box::new(Expr::Int(10))),
        Expr::Set("b".into(), Box::new(Expr::Int(20))),
        Expr::Set("c".into(), Box::new(Expr::Int(30))),
        Expr::Binary(
            Operation::Add,
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Var("a".into())),
                Box::new(Expr::Var("b".into())),
            )),
            Box::new(Expr::Var("c".into())),
        )
    ]);
    
    let result = eva.eval(block_expr).unwrap();
    assert_eq!(result, Value::Int(60)); // 10 + 20 + 30
    
    // Variables a, b, c should not exist in global scope
    let err = eva.eval(Expr::Var("a".into())).unwrap_err();
    assert_eq!(err, "undefined variable");
}

#[test]
fn block_empty_returns_null() {
    let eva = Eva::new();
    
    // Empty block should return Null
    let block_expr = Expr::Block(vec![]);
    
    let result = eva.eval(block_expr).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn block_variables_persist_across_expressions() {
    let eva = Eva::new();
    
    // Test that variables set in one expression are available in the next
    let block_expr = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(5))),
        Expr::Set("y".into(), Box::new(Expr::Int(10))),
        // This should be able to access both x and y
        Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Var("x".into())),
            Box::new(Expr::Var("y".into())),
        )
    ]);
    
    let result = eva.eval(block_expr).unwrap();
    assert_eq!(result, Value::Int(50)); // 5 * 10
}

#[test]
fn nested_block_accesses_parent_variable() {
    let eva = Eva::new();
    
    // Parent block sets a variable, child block uses it
    let child_block = Expr::Block(vec![
        Expr::Set("y".into(), Box::new(Expr::Int(20))),
        // Access parent's x and child's y
        Expr::Binary(
            Operation::Add,
            Box::new(Expr::Var("x".into())), // from parent
            Box::new(Expr::Var("y".into())), // from child
        )
    ]);
    
    let parent_block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        child_block,
    ]);
    
    let result = eva.eval(parent_block).unwrap();
    assert_eq!(result, Value::Int(30)); // 10 + 20
}

#[test]
fn nested_block_shadows_parent_variable() {
    let eva = Eva::new();
    
    // Both parent and child have the same variable name
    let child_block = Expr::Block(vec![
        // Shadow parent's x with child's x
        Expr::Set("x".into(), Box::new(Expr::Int(100))),
        // This should use child's x (100), not parent's x (10)
        Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Var("x".into())),
            Box::new(Expr::Int(2)),
        )
    ]);
    
    let parent_block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        child_block, // This will return 200
        // After child block, parent's x should still be 10
        Expr::Var("x".into()),
    ]);
    
    let result = eva.eval(parent_block).unwrap();
    assert_eq!(result, Value::Int(10)); // parent's x is unchanged
}

#[test]
fn deeply_nested_blocks_with_shadowing() {
    let eva = Eva::new();
    
    // Global x = 1
    eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();
    
    // Level 3 (innermost): x = 30
    let level3_block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(30))),
        Expr::Var("x".into()), // Should be 30
    ]);
    
    // Level 2 (middle): x = 20, then run level3
    let level2_block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(20))),
        level3_block, // Returns 30
        Expr::Var("x".into()), // Should still be 20
    ]);
    
    // Level 1 (outer): x = 10, then run level2
    let level1_block = Expr::Block(vec![
        Expr::Set("x".into(), Box::new(Expr::Int(10))),
        level2_block, // Returns 20
        Expr::Var("x".into()), // Should still be 10
    ]);
    
    let result = eva.eval(level1_block).unwrap();
    assert_eq!(result, Value::Int(10)); // level1's x is unchanged
    
    // Global x should still be 1
    let global_x = eva.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(global_x, Value::Int(1));
}

#[test]
fn block_with_variable_referencing_outer_scope() {
    let eva = Eva::new();
    
    // ['begin',
    //   ['var', 'value', 10],
    //   ['var', 'result', ['begin',
    //     ['var', 'x', ['+', 'value', 10]],
    //     'x'
    //   ]],
    //   'result'
    // ]
    
    let expr = Expr::Block(vec![
        // var value = 10
        Expr::Set("value".into(), Box::new(Expr::Int(10))),
        
        // var result = begin
        //   var x = value + 10
        //   x
        // end
        Expr::Set(
            "result".into(),
            Box::new(Expr::Block(vec![
                Expr::Set(
                    "x".into(),
                    Box::new(Expr::Binary(
                        Operation::Add,
                        Box::new(Expr::Var("value".into())),
                        Box::new(Expr::Int(10)),
                    )),
                ),
                Expr::Var("x".into()),
            ])),
        ),
        
        // result
        Expr::Var("result".into()),
    ]);
    
    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(20));
}
