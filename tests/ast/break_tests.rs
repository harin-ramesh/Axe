use axe::{Axe, EvalSignal, Literal, Parser, Value};

// Helper function to parse and evaluate code
fn eval(code: &str) -> Result<Value, EvalSignal> {
    let mut parser = Parser::new(code);
    let program = parser
        .parse()
        .map_err(|e| EvalSignal::Error(e.to_string()))?;
    let mut axe = Axe::new();
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
// Basic Break Tests - While Loop
// ============================================================================

#[test]
fn break_while_loop_immediately() {
    let code = r#"
        let count = 0;
        while (true) {
            break;
            count = count + 1;
        }
        count;
    "#;
    assert_eq!(eval_int(code), 0);
}

#[test]
fn break_while_loop_after_condition() {
    let code = r#"
        let i = 0;
        while (true) {
            if (i >= 5) {
                break;
            }
            i = i + 1;
        }
        i;
    "#;
    assert_eq!(eval_int(code), 5);
}

#[test]
fn break_while_loop_sum() {
    let code = r#"
        let sum = 0;
        let i = 1;
        while (true) {
            if (i > 10) {
                break;
            }
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;
    // 1+2+3+...+10 = 55
    assert_eq!(eval_int(code), 55);
}

#[test]
fn break_while_with_normal_condition() {
    let code = r#"
        let i = 0;
        let result = 0;
        while (i < 100) {
            if (i == 7) {
                break;
            }
            result = result + i;
            i = i + 1;
        }
        result;
    "#;
    // 0+1+2+3+4+5+6 = 21
    assert_eq!(eval_int(code), 21);
}

// ============================================================================
// Break in For Loop Tests
// ============================================================================

#[test]
fn break_for_loop() {
    let code = r#"
        let sum = 0;
        for i in range(100) {
            if (i >= 5) {
                break;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // 0+1+2+3+4 = 10
    assert_eq!(eval_int(code), 10);
}

#[test]
fn break_for_loop_first_iteration() {
    let code = r#"
        let count = 0;
        for i in range(10) {
            break;
            count = count + 1;
        }
        count;
    "#;
    assert_eq!(eval_int(code), 0);
}

#[test]
fn break_for_loop_conditional() {
    let code = r#"
        let last = 0;
        for i in range(1, 100) {
            if (i * i > 50) {
                break;
            }
            last = i;
        }
        last;
    "#;
    // 7*7=49 <= 50, 8*8=64 > 50, so last = 7
    assert_eq!(eval_int(code), 7);
}

// ============================================================================
// Nested Break Tests
// ============================================================================

#[test]
fn break_inner_loop_only() {
    let code = r#"
        let outer_count = 0;
        let inner_count = 0;
        for i in range(3) {
            outer_count = outer_count + 1;
            for j in range(100) {
                if (j >= 2) {
                    break;
                }
                inner_count = inner_count + 1;
            }
        }
        outer_count * 100 + inner_count;
    "#;
    // outer runs 3 times, inner runs 2 iterations each (j=0,1 then break at j=2)
    // outer_count = 3, inner_count = 6
    // 3*100 + 6 = 306
    assert_eq!(eval_int(code), 306);
}

#[test]
fn break_outer_while_with_inner_for() {
    let code = r#"
        let total = 0;
        let round = 0;
        while (true) {
            if (round >= 3) {
                break;
            }
            for i in range(5) {
                total = total + 1;
            }
            round = round + 1;
        }
        total;
    "#;
    // 3 rounds * 5 iterations = 15
    assert_eq!(eval_int(code), 15);
}

// ============================================================================
// Break with State Preservation Tests
// ============================================================================

#[test]
fn break_preserves_variables() {
    let code = r#"
        let result = 0;
        for i in range(10) {
            result = i;
            if (i == 5) {
                break;
            }
        }
        result;
    "#;
    assert_eq!(eval_int(code), 5);
}

#[test]
fn break_in_while_preserves_state() {
    let code = r#"
        let x = 100;
        let i = 0;
        while (i < 10) {
            x = x - 1;
            i = i + 1;
            if (x == 95) {
                break;
            }
        }
        x;
    "#;
    assert_eq!(eval_int(code), 95);
}

// ============================================================================
// Break with Functions Tests
// ============================================================================

#[test]
fn break_inside_function() {
    let code = r#"
        fn countTo(limit) {
            let count = 0;
            let i = 0;
            while (true) {
                if (i >= limit) {
                    break;
                }
                count = count + 1;
                i = i + 1;
            }
            return count;
        }
        countTo(7);
    "#;
    assert_eq!(eval_int(code), 7);
}

#[test]
fn break_vs_return_in_function() {
    let code = r#"
        fn firstMultiple(base, limit) {
            for i in range(1, 100) {
                if (i * base > limit) {
                    return i;
                }
            }
            return -1;
        }
        firstMultiple(7, 50);
    "#;
    // 8*7=56 > 50
    assert_eq!(eval_int(code), 8);
}

// ============================================================================
// Break Edge Cases
// ============================================================================

#[test]
fn break_empty_while_body() {
    let code = r#"
        let x = 0;
        while (true) {
            x = x + 1;
            if (x > 0) {
                break;
            }
        }
        x;
    "#;
    assert_eq!(eval_int(code), 1);
}

#[test]
fn break_after_multiple_iterations() {
    let code = r#"
        let product = 1;
        for i in range(1, 20) {
            product = product * i;
            if (product > 100) {
                break;
            }
        }
        product;
    "#;
    // 1*1=1, 1*2=2, 2*3=6, 6*4=24, 24*5=120 > 100 -> break
    assert_eq!(eval_int(code), 120);
}
