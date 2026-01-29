use axe::{Axe, Literal, Parser, Value};

// Helper function to parse and evaluate code
fn eval(code: &str) -> Result<Value, &'static str> {
    let mut parser = Parser::new(code);
    let program = parser.parse().map_err(|_| "parse error")?;
    let axe = Axe::new();
    axe.run(program)
}

// Helper to get int value from evaluation
fn eval_int(code: &str) -> i64 {
    match eval(code) {
        Ok(Value::Literal(Literal::Int(n))) => n,
        other => panic!("Expected Int, got {:?}", other),
    }
}

// ============================================================================
// Basic For Loop Tests
// ============================================================================

#[test]
fn for_loop_basic_range() {
    let code = r#"
        let sum = 0;
        for i in range(5) {
            sum = sum + 1;
        }
        sum;
    "#;
    assert_eq!(eval_int(code), 5);
}

#[test]
fn for_loop_range_with_start_end() {
    let code = r#"
        let sum = 0;
        for i in range(1, 6) {
            sum = sum + i;
        }
        sum;
    "#;
    // 1 + 2 + 3 + 4 + 5 = 15
    assert_eq!(eval_int(code), 15);
}

#[test]
fn for_loop_range_empty() {
    let code = r#"
        let count = 0;
        for i in range(0) {
            count = count + 1;
        }
        count;
    "#;
    assert_eq!(eval_int(code), 0);
}

#[test]
fn for_loop_sum_1_to_10() {
    let code = r#"
        let sum = 0;
        for n in range(1, 11) {
            sum = sum + n;
        }
        sum;
    "#;
    assert_eq!(eval_int(code), 55);
}

#[test]
fn for_loop_sum_of_squares() {
    let code = r#"
        let sum = 0;
        for i in range(1, 6) {
            sum = sum + i * i;
        }
        sum;
    "#;
    // 1 + 4 + 9 + 16 + 25 = 55
    assert_eq!(eval_int(code), 55);
}

// ============================================================================
// Nested For Loop Tests
// ============================================================================

#[test]
fn for_loop_nested() {
    let code = r#"
        let count = 0;
        for i in range(3) {
            for j in range(4) {
                count = count + 1;
            }
        }
        count;
    "#;
    assert_eq!(eval_int(code), 12);
}

#[test]
fn for_loop_nested_sum() {
    let code = r#"
        let sum = 0;
        for i in range(1, 4) {
            for j in range(1, 4) {
                sum = sum + i * j;
            }
        }
        sum;
    "#;
    // (1*1 + 1*2 + 1*3) + (2*1 + 2*2 + 2*3) + (3*1 + 3*2 + 3*3)
    // = 6 + 12 + 18 = 36
    assert_eq!(eval_int(code), 36);
}

#[test]
fn for_loop_triple_nested() {
    let code = r#"
        let count = 0;
        for i in range(2) {
            for j in range(3) {
                for k in range(4) {
                    count = count + 1;
                }
            }
        }
        count;
    "#;
    assert_eq!(eval_int(code), 24);
}

// ============================================================================
// For Loop with Expressions Tests
// ============================================================================

#[test]
fn for_loop_factorial() {
    let code = r#"
        let result = 1;
        for i in range(1, 6) {
            result = result * i;
        }
        result;
    "#;
    // 5! = 120
    assert_eq!(eval_int(code), 120);
}

#[test]
fn for_loop_power() {
    let code = r#"
        let base = 2;
        let result = 1;
        for i in range(10) {
            result = result * base;
        }
        result;
    "#;
    // 2^10 = 1024
    assert_eq!(eval_int(code), 1024);
}

#[test]
fn for_loop_with_conditional() {
    let code = r#"
        let even_sum = 0;
        for i in range(1, 11) {
            if (i % 2 == 0) {
                even_sum = even_sum + i;
            } else {
                even_sum = even_sum;
            }
        }
        even_sum;
    "#;
    // 2 + 4 + 6 + 8 + 10 = 30
    assert_eq!(eval_int(code), 30);
}

#[test]
fn for_loop_with_odd_sum() {
    let code = r#"
        let odd_sum = 0;
        for i in range(1, 11) {
            if (i % 2 == 1) {
                odd_sum = odd_sum + i;
            } else {
                odd_sum = odd_sum;
            }
        }
        odd_sum;
    "#;
    // 1 + 3 + 5 + 7 + 9 = 25
    assert_eq!(eval_int(code), 25);
}

// ============================================================================
// For Loop with Functions Tests
// ============================================================================

#[test]
fn for_loop_with_function_call() {
    let code = r#"
        fn square(x) {
            x * x;
        }
        
        let sum = 0;
        for i in range(1, 6) {
            sum = sum + square(i);
        }
        sum;
    "#;
    // 1 + 4 + 9 + 16 + 25 = 55
    assert_eq!(eval_int(code), 55);
}

#[test]
fn for_loop_inside_function() {
    let code = r#"
        fn sumRange(n) {
            let total = 0;
            for i in range(1, n + 1) {
                total = total + i;
            }
            total;
        }
        
        sumRange(10);
    "#;
    assert_eq!(eval_int(code), 55);
}

#[test]
fn for_loop_function_factorial() {
    let code = r#"
        fn factorial(n) {
            let result = 1;
            for i in range(1, n + 1) {
                result = result * i;
            }
            result;
        }
        
        factorial(5);
    "#;
    assert_eq!(eval_int(code), 120);
}

// ============================================================================
// For Loop Variable Scope Tests
// ============================================================================

#[test]
fn for_loop_variable_scope() {
    let code = r#"
        let i = 100;
        for i in range(5) {
            i;
        }
        i;
    "#;
    // After loop, outer i should still be 100
    // Note: This tests that loop variable doesn't leak
    assert_eq!(eval_int(code), 100);
}

#[test]
fn for_loop_modifies_outer_variable() {
    let code = r#"
        let total = 0;
        for i in range(5) {
            total = total + i;
        }
        total;
    "#;
    // 0 + 1 + 2 + 3 + 4 = 10
    assert_eq!(eval_int(code), 10);
}

// ============================================================================
// For Loop Edge Cases
// ============================================================================

#[test]
fn for_loop_single_iteration() {
    let code = r#"
        let count = 0;
        for i in range(1) {
            count = count + 1;
        }
        count;
    "#;
    assert_eq!(eval_int(code), 1);
}

#[test]
fn for_loop_large_range() {
    let code = r#"
        let sum = 0;
        for i in range(100) {
            sum = sum + 1;
        }
        sum;
    "#;
    assert_eq!(eval_int(code), 100);
}

#[test]
fn for_loop_product() {
    let code = r#"
        let product = 1;
        for i in range(1, 6) {
            product = product * i;
        }
        product;
    "#;
    assert_eq!(eval_int(code), 120);
}

// ============================================================================
// For Loop with Different Data Types
// ============================================================================

#[test]
fn for_loop_string_concatenation() {
    let code = r#"
        let result = "";
        for i in range(3) {
            result = result.concat("a");
        }
        result;
    "#;
    match eval(code) {
        Ok(Value::Literal(Literal::Str(s))) => assert_eq!(s, "aaa"),
        other => panic!("Expected Str, got {:?}", other),
    }
}

#[test]
fn for_loop_build_list() {
    let code = r#"
        let list = [];
        for i in range(3) {
            list = list.push(i);
        }
        list.len();
    "#;
    assert_eq!(eval_int(code), 3);
}

// ============================================================================
// Complex For Loop Tests
// ============================================================================

#[test]
fn for_loop_fibonacci() {
    let code = r#"
        let a = 0;
        let b = 1;
        for i in range(10) {
            let temp = a + b;
            a = b;
            b = temp;
        }
        a;
    "#;
    // fib(10) = 55
    assert_eq!(eval_int(code), 55);
}

#[test]
fn for_loop_countdown() {
    let code = r#"
        let values = [];
        for i in range(5) {
            values = values.push(5 - i);
        }
        values.get(0) + values.get(4);
    "#;
    // First: 5, Last: 1
    assert_eq!(eval_int(code), 6);
}

#[test]
fn for_loop_with_multiplication_table() {
    let code = r#"
        let sum = 0;
        for i in range(1, 6) {
            for j in range(1, 6) {
                if (i == j) {
                    sum = sum + i * j;
                } else {
                    sum = sum;
                }
            }
        }
        sum;
    "#;
    // 1*1 + 2*2 + 3*3 + 4*4 + 5*5 = 1 + 4 + 9 + 16 + 25 = 55
    assert_eq!(eval_int(code), 55);
}
