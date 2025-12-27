use axe::{Axe, Parser};

#[test]
fn test_print_exists() {
    let axe = Axe::new();
    let mut parser = Parser::new("(print \"Hello\")").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr);
    
    // Print function should return null and not error
    assert!(result.is_ok());
    assert_eq!(format!("{:?}", result.unwrap()), "Null");
}

#[test]
fn test_print_multiple_args() {
    let axe = Axe::new();
    let mut parser = Parser::new("(print \"Hello\" \"World\" 42)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr);
    
    // Print function should return null and not error
    assert!(result.is_ok());
    assert_eq!(format!("{:?}", result.unwrap()), "Null");
}

#[test]
fn test_print_with_variables() {
    let axe = Axe::new();
    
    // Set a variable
    let mut parser = Parser::new("(let x 10)").unwrap();
    let expr = parser.parse().unwrap();
    axe.eval(expr).unwrap();
    
    // Print the variable
    let mut parser = Parser::new("(print \"x is\" x)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr);
    
    assert!(result.is_ok());
    assert_eq!(format!("{:?}", result.unwrap()), "Null");
}
