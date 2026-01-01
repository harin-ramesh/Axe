use axe::{Axe, Parser};

#[test]
fn test_increment_basic() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let i 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(++ i)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("i").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    assert_eq!(result.to_string(), "1");
}

#[test]
fn test_decrement_basic() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let i 10)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(-- i)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("i").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    assert_eq!(result.to_string(), "9");
}

#[test]
fn test_multiple_increments() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let x 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(++ x)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(++ x)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(++ x)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("x").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_increment_in_while_loop() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let count 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(let i 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(while (< i 5) (let count (+ count 1)) (++ i))").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("count").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    assert_eq!(result.to_string(), "5");
}

#[test]
fn test_decrement_in_while_loop() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let count 10)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(while (> count 0) (-- count))").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("count").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    assert_eq!(result.to_string(), "0");
}

#[test]
fn test_increment_decrement_combination() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let x 5)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(++ x)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(++ x)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(-- x)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("x").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    assert_eq!(result.to_string(), "6");
}
