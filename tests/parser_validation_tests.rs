use axe::Parser;

#[test]
fn test_empty_while_body_error() {
    let mut parser = Parser::new("(while (< x 5))").unwrap();
    let result = parser.parse();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "While loop requires non-empty body");
}

#[test]
fn test_empty_if_then_branch_error() {
    let mut parser = Parser::new("(if true (begin) 10)").unwrap();
    let result = parser.parse();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "If statement requires non-empty then branch");
}

#[test]
fn test_empty_if_else_branch_error() {
    let mut parser = Parser::new("(if true 10 (begin))").unwrap();
    let result = parser.parse();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "If statement requires non-empty else branch");
}

#[test]
fn test_valid_while_with_body() {
    let mut parser = Parser::new("(while (< x 5) (let x (+ x 1)))").unwrap();
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_valid_if_with_branches() {
    let mut parser = Parser::new("(if true 10 20)").unwrap();
    let result = parser.parse();
    assert!(result.is_ok());
}
