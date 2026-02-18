use axe::{Axe, Context, EvalSignal, Literal, Parser, Value};

// Helper function to parse and evaluate code
fn eval(code: &str) -> Result<Value, EvalSignal> {
    let context = Context::new();
    let mut parser = Parser::new(code, &context);
    let program = parser
        .parse()
        .map_err(|e| EvalSignal::Error(e.to_string()))?;
    let mut axe = Axe::new(&context);
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
// Basic Continue Tests - While Loop
// ============================================================================

#[test]
fn continue_skips_rest_of_iteration() {
    let code = r#"
        let sum = 0;
        let i = 0;
        while (i < 10) {
            i = i + 1;
            if (i % 2 == 0) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // Odd numbers 1+3+5+7+9 = 25
    assert_eq!(eval_int(code), 25);
}

#[test]
fn continue_while_skip_multiples_of_3() {
    let code = r#"
        let count = 0;
        let i = 0;
        while (i < 10) {
            i = i + 1;
            if (i % 3 == 0) {
                continue;
            }
            count = count + 1;
        }
        count;
    "#;
    // Skip 3, 6, 9 -> count = 10 - 3 = 7
    assert_eq!(eval_int(code), 7);
}

#[test]
fn continue_while_accumulate_non_skipped() {
    let code = r#"
        let result = 0;
        let i = 0;
        while (i < 5) {
            i = i + 1;
            if (i == 3) {
                continue;
            }
            result = result + i * 10;
        }
        result;
    "#;
    // 10 + 20 + (skip 30) + 40 + 50 = 120
    assert_eq!(eval_int(code), 120);
}

// ============================================================================
// Continue in For Loop Tests
// ============================================================================

#[test]
fn continue_for_loop_skip_even() {
    let code = r#"
        let sum = 0;
        for i in range(1, 11) {
            if (i % 2 == 0) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // 1+3+5+7+9 = 25
    assert_eq!(eval_int(code), 25);
}

#[test]
fn continue_for_loop_skip_odd() {
    let code = r#"
        let sum = 0;
        for i in range(1, 11) {
            if (i % 2 == 1) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // 2+4+6+8+10 = 30
    assert_eq!(eval_int(code), 30);
}

#[test]
fn continue_for_loop_skip_specific() {
    let code = r#"
        let sum = 0;
        for i in range(1, 6) {
            if (i == 3) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // 1+2+4+5 = 12
    assert_eq!(eval_int(code), 12);
}

#[test]
fn continue_for_loop_count_not_skipped() {
    let code = r#"
        let count = 0;
        for i in range(20) {
            if (i % 5 == 0) {
                continue;
            }
            count = count + 1;
        }
        count;
    "#;
    // Skip 0, 5, 10, 15 -> 20 - 4 = 16
    assert_eq!(eval_int(code), 16);
}

// ============================================================================
// Nested Continue Tests
// ============================================================================

#[test]
fn continue_inner_loop_only() {
    let code = r#"
        let total = 0;
        for i in range(3) {
            for j in range(5) {
                if (j == 2) {
                    continue;
                }
                total = total + 1;
            }
        }
        total;
    "#;
    // Each inner loop runs 5 times, skips j=2, so 4 per outer iteration
    // 3 * 4 = 12
    assert_eq!(eval_int(code), 12);
}

#[test]
fn continue_nested_while_loops() {
    let code = r#"
        let sum = 0;
        let i = 0;
        while (i < 3) {
            i = i + 1;
            let j = 0;
            while (j < 3) {
                j = j + 1;
                if (j == 2) {
                    continue;
                }
                sum = sum + 1;
            }
        }
        sum;
    "#;
    // Each inner: j=1 (count), j=2 (skip), j=3 (count) = 2 per outer
    // 3 * 2 = 6
    assert_eq!(eval_int(code), 6);
}

// ============================================================================
// Continue with Break Tests
// ============================================================================

#[test]
fn continue_and_break_together() {
    let code = r#"
        let sum = 0;
        for i in range(100) {
            if (i % 2 == 0) {
                continue;
            }
            if (i > 10) {
                break;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // Odd numbers up to 10: 1+3+5+7+9 = 25
    // i=11 is odd, > 10 -> break
    assert_eq!(eval_int(code), 25);
}

#[test]
fn continue_and_break_while() {
    let code = r#"
        let result = 0;
        let i = 0;
        while (true) {
            i = i + 1;
            if (i > 20) {
                break;
            }
            if (i % 3 != 0) {
                continue;
            }
            result = result + i;
        }
        result;
    "#;
    // Multiples of 3 up to 20: 3+6+9+12+15+18 = 63
    assert_eq!(eval_int(code), 63);
}

// ============================================================================
// Continue in Functions Tests
// ============================================================================

#[test]
fn continue_inside_function() {
    let code = r#"
        fn sumOdds(n) {
            let total = 0;
            for i in range(1, n + 1) {
                if (i % 2 == 0) {
                    continue;
                }
                total = total + i;
            }
            return total;
        }
        sumOdds(10);
    "#;
    // 1+3+5+7+9 = 25
    assert_eq!(eval_int(code), 25);
}

#[test]
fn continue_with_return() {
    let code = r#"
        fn countNonMultiples(n, divisor) {
            let count = 0;
            for i in range(1, n + 1) {
                if (i % divisor == 0) {
                    continue;
                }
                count = count + 1;
            }
            return count;
        }
        countNonMultiples(20, 4);
    "#;
    // Multiples of 4 in 1..20: 4,8,12,16,20 = 5 -> 20-5=15
    assert_eq!(eval_int(code), 15);
}

// ============================================================================
// Continue Edge Cases
// ============================================================================

#[test]
fn continue_every_iteration() {
    let code = r#"
        let sum = 0;
        for i in range(10) {
            continue;
            sum = sum + i;
        }
        sum;
    "#;
    // All iterations are skipped, sum stays 0
    assert_eq!(eval_int(code), 0);
}

#[test]
fn continue_never_triggered() {
    let code = r#"
        let sum = 0;
        for i in range(1, 6) {
            if (false) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // 1+2+3+4+5 = 15
    assert_eq!(eval_int(code), 15);
}

#[test]
fn continue_on_last_iteration() {
    let code = r#"
        let sum = 0;
        for i in range(5) {
            if (i == 4) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    // 0+1+2+3 = 6
    assert_eq!(eval_int(code), 6);
}
