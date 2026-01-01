use axe::{Axe, Parser};

#[test]
fn test_for_loop_basic() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let sum 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(for (let i 0) (< i 5) (++ i) (let sum (+ sum i)))").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("sum").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    // sum = 0 + 1 + 2 + 3 + 4 = 10
    assert_eq!(result.to_string(), "10");
}

#[test]
fn test_for_loop_countdown() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let count 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(for (let i 10) (> i 0) (-- i) (let count (+ count 1)))").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("count").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    assert_eq!(result.to_string(), "10");
}

#[test]
fn test_for_loop_with_array() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let numbers (list 10 20 30 40))").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(let sum 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(for (let i 0) (< i (len numbers)) (++ i) (let sum (+ sum (get numbers i))))").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("sum").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    // sum = 10 + 20 + 30 + 40 = 100
    assert_eq!(result.to_string(), "100");
}

#[test]
fn test_nested_for_loops() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let product 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let code = "(for (let i 1) (<= i 3) (++ i) (for (let j 1) (<= j 3) (++ j) (let product (* i j))))";
    let mut parser = Parser::new(code).unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("product").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    // Last value: 3 * 3 = 9
    assert_eq!(result.to_string(), "9");
}

#[test]
fn test_for_loop_empty_iterations() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let count 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(for (let i 0) (< i 0) (++ i) (let count (+ count 1)))").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("count").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    // Loop never executes
    assert_eq!(result.to_string(), "0");
}

#[test]
fn test_for_loop_with_multiple_statements() {
    let axe = Axe::new();
    
    let mut parser = Parser::new("(let sum 0)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(let product 1)").unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let code = "(for (let i 1) (<= i 5) (++ i) (let sum (+ sum i)) (let product (* product i)))";
    let mut parser = Parser::new(code).unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    // Check sum
    let mut parser = Parser::new("sum").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    assert_eq!(result.to_string(), "15"); // 1+2+3+4+5
    
    // Check product
    let mut parser = Parser::new("product").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    assert_eq!(result.to_string(), "120"); // 1*2*3*4*5 (factorial)
}

#[test]
fn test_for_loop_in_function() {
    let axe = Axe::new();
    
    let code = "(fn sum_to_n (n) (let sum 0) (for (let i 1) (<= i n) (++ i) (let sum (+ sum i))) sum)";
    let mut parser = Parser::new(code).unwrap();
    axe.eval(parser.parse().unwrap()).unwrap();
    
    let mut parser = Parser::new("(sum_to_n 10)").unwrap();
    let result = axe.eval(parser.parse().unwrap()).unwrap();
    
    // sum = 1+2+3+...+10 = 55
    assert_eq!(result.to_string(), "55");
}
