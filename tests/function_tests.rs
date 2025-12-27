use axe::{Axe, Parser, Value};

#[test]
fn simple_function_definition_and_call() {
    let axe = Axe::new();
    
    // Define a function: (set add2 (fn (x) (+ x 2)))
    let mut parser = Parser::new("(set add2 (fn (x) (+ x 2)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // Call the function: (add2 5)
    let mut parser = Parser::new("(add2 5)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(7));
}

#[test]
fn function_with_multiple_parameters() {
    let axe = Axe::new();
    
    // (set add (fn (x y) (+ x y)))
    let mut parser = Parser::new("(set add (fn (x y) (+ x y)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (add 10 20)
    let mut parser = Parser::new("(add 10 20)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(30));
}

#[test]
fn function_with_no_parameters() {
    let axe = Axe::new();
    
    // (set get42 (fn () 42))
    let mut parser = Parser::new("(set get42 (fn () 42))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (get42)
    let mut parser = Parser::new("(get42)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(42));
}

#[test]
fn function_with_multiple_expressions_in_body() {
    let axe = Axe::new();
    
    // (set calc (fn (x) (set y (* x 2)) (+ y 3)))
    let mut parser = Parser::new("(set calc (fn (x) (set y (* x 2)) (+ y 3)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (calc 5)
    let mut parser = Parser::new("(calc 5)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(13)); // (5 * 2) + 3 = 13
}

#[test]
fn function_capturing_closure_variable() {
    let axe = Axe::new();
    
    // (set x 10)
    let mut parser = Parser::new("(set x 10)").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (set addX (fn (y) (+ x y)))
    let mut parser = Parser::new("(set addX (fn (y) (+ x y)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (addX 5)
    let mut parser = Parser::new("(addX 5)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(15)); // 10 + 5
}

#[test]
fn nested_function_calls() {
    let axe = Axe::new();
    
    // (set double (fn (x) (* x 2)))
    let mut parser = Parser::new("(set double (fn (x) (* x 2)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (set add3 (fn (x) (+ x 3)))
    let mut parser = Parser::new("(set add3 (fn (x) (+ x 3)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (double (add3 5))
    let mut parser = Parser::new("(double (add3 5))").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(16)); // (5 + 3) * 2 = 16
}

#[test]
fn function_call_with_wrong_argument_count() {
    let axe = Axe::new();
    
    // (set add (fn (x y) (+ x y)))
    let mut parser = Parser::new("(set add (fn (x y) (+ x y)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (add 10) - should fail
    let mut parser = Parser::new("(add 10)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "argument count mismatch");
}

#[test]
fn calling_undefined_function() {
    let axe = Axe::new();
    
    // (foo 10)
    let mut parser = Parser::new("(foo 10)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "undefined function");
}

#[test]
fn calling_non_function_value() {
    let axe = Axe::new();
    
    // (set x 42)
    let mut parser = Parser::new("(set x 42)").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (x 10) - should fail
    let mut parser = Parser::new("(x 10)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "not a function");
}

#[test]
fn recursive_function() {
    let axe = Axe::new();
    
    // (set factorial (fn (n) (if (<= n 1) 1 (* n (factorial (- n 1))))))
    let mut parser = Parser::new("(set factorial (fn (n) (if (<= n 1) 1 (* n (factorial (- n 1))))))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (factorial 5)
    let mut parser = Parser::new("(factorial 5)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(120)); // 5! = 120
}

#[test]
fn function_in_condition() {
    let axe = Axe::new();
    
    // (set isPositive (fn (x) (> x 0)))
    let mut parser = Parser::new("(set isPositive (fn (x) (> x 0)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (if (isPositive 5) "yes" "no")
    let mut parser = Parser::new("(if (isPositive 5) \"yes\" \"no\")").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Str("yes".to_string()));
}

#[test]
fn function_returning_function() {
    let axe = Axe::new();
    
    // (set makeAdder (fn (x) (fn (y) (+ x y))))
    let mut parser = Parser::new("(set makeAdder (fn (x) (fn (y) (+ x y))))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (set add5 (makeAdder 5))
    let mut parser = Parser::new("(set add5 (makeAdder 5))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (add5 10)
    let mut parser = Parser::new("(add5 10)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(15)); // 5 + 10
}

#[test]
fn function_with_invalid_parameter_name() {
    let axe = Axe::new();
    
    // (set bad (fn (hello-world) (+ hello-world 1))) - parameter name contains hyphen
    let mut parser = Parser::new("(set bad (fn (hello-world) (+ hello-world 1)))").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "invalid parameter name");
}

#[test]
fn function_scope_isolation() {
    let axe = Axe::new();
    
    // (set x 10)
    let mut parser = Parser::new("(set x 10)").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (set changeX (fn () (set x 100)))
    let mut parser = Parser::new("(set changeX (fn () (set x 100)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (changeX)
    let mut parser = Parser::new("(changeX)").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // x should still be 10 (function creates new scope)
    let mut parser = Parser::new("x").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(10));
}

#[test]
fn function_modifying_outer_variable() {
    let axe = Axe::new();
    
    // (set x 10)
    let mut parser = Parser::new("(set x 10)").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (set modifyX (fn () (assign x 100)))
    let mut parser = Parser::new("(set modifyX (fn () (assign x 100)))").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // (modifyX)
    let mut parser = Parser::new("(modifyX)").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // x should now be 100 (assign modifies outer scope)
    let mut parser = Parser::new("x").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    
    assert_eq!(result, Value::Int(100));
}
