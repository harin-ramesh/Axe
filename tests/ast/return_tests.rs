use axe::{Axe, Context, EvalSignal, Literal, Parser, Value};

// Helper function to parse and evaluate code
fn eval(code: &str) -> Result<(Value, Context), EvalSignal> {
    let context = Context::new();
    let mut parser = Parser::new(code, &context);
    let program = parser
        .parse()
        .map_err(|e| EvalSignal::Error(e.to_string()))?;
    let mut axe = Axe::new(&context);
    let value = axe.run(program)?;
    Ok((value, context))
}

// Helper to get int value from evaluation
fn eval_int(code: &str) -> i64 {
    match eval(code) {
        Ok((Value::Literal(Literal::Int(n)), _)) => n,
        other => panic!("Expected Int, got {:?}", other),
    }
}

// Helper to get bool value from evaluation
fn eval_bool(code: &str) -> bool {
    match eval(code) {
        Ok((Value::Literal(Literal::Bool(b)), _)) => b,
        other => panic!("Expected Bool, got {:?}", other),
    }
}

// ============================================================================
// Basic Return Tests
// ============================================================================

#[test]
fn return_simple_value() {
    let code = r#"
        fn five() {
            return 5;
        }
        five();
    "#;
    assert_eq!(eval_int(code), 5);
}

#[test]
fn return_expression() {
    let code = r#"
        fn add(a, b) {
            return a + b;
        }
        add(3, 4);
    "#;
    assert_eq!(eval_int(code), 7);
}

#[test]
fn return_string() {
    let code = r#"
        fn greet() {
            return "hello";
        }
        greet();
    "#;
    match eval(code) {
        Ok((Value::Literal(Literal::Str(s)), ctx)) => assert_eq!(ctx.resolve(s), "hello"),
        other => panic!("Expected Str, got {:?}", other),
    }
}

#[test]
fn return_boolean() {
    let code = r#"
        fn isTrue() {
            return true;
        }
        isTrue();
    "#;
    assert_eq!(eval_bool(code), true);
}

#[test]
fn return_null() {
    let code = r#"
        fn nothing() {
            return null;
        }
        nothing();
    "#;
    match eval(code) {
        Ok((Value::Literal(Literal::Null), _)) => {}
        other => panic!("Expected Null, got {:?}", other),
    }
}

// ============================================================================
// Early Return Tests
// ============================================================================

#[test]
fn return_early_from_if() {
    let code = r#"
        fn abs(x) {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        abs(-42);
    "#;
    assert_eq!(eval_int(code), 42);
}

#[test]
fn return_early_positive_path() {
    let code = r#"
        fn abs(x) {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        abs(42);
    "#;
    assert_eq!(eval_int(code), 42);
}

#[test]
fn return_early_skips_remaining() {
    let code = r#"
        fn test() {
            return 1;
            return 2;
        }
        test();
    "#;
    assert_eq!(eval_int(code), 1);
}

#[test]
fn return_from_if_else() {
    let code = r#"
        fn sign(x) {
            if (x > 0) {
                return 1;
            } else {
                if (x < 0) {
                    return -1;
                } else {
                    return 0;
                }
            }
        }
        sign(-5) + sign(0) + sign(10);
    "#;
    // -1 + 0 + 1 = 0
    assert_eq!(eval_int(code), 0);
}

// ============================================================================
// Return in Loops Tests
// ============================================================================

#[test]
fn return_from_while_loop() {
    let code = r#"
        fn findFirst() {
            let i = 0;
            while (i < 100) {
                if (i == 7) {
                    return i;
                }
                i = i + 1;
            }
            return -1;
        }
        findFirst();
    "#;
    assert_eq!(eval_int(code), 7);
}

#[test]
fn return_from_for_loop() {
    let code = r#"
        fn findInRange() {
            for i in range(100) {
                if (i * i > 50) {
                    return i;
                }
            }
            return -1;
        }
        findInRange();
    "#;
    // 8*8 = 64 > 50
    assert_eq!(eval_int(code), 8);
}

#[test]
fn return_from_nested_loops() {
    let code = r#"
        fn findPair() {
            for i in range(1, 10) {
                for j in range(1, 10) {
                    if (i * j == 12) {
                        return i + j;
                    }
                }
            }
            return -1;
        }
        findPair();
    "#;
    // First pair: i=2, j=6 -> 2+6 = 8  (or i=1 won't find it since 1*j=12 means j=12 > 9)
    // Actually i=2, j=6 -> return 8
    assert_eq!(eval_int(code), 8);
}

// ============================================================================
// Recursive Return Tests
// ============================================================================

#[test]
fn return_recursive_factorial() {
    let code = r#"
        fn factorial(n) {
            if (n <= 1) {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }
        factorial(5);
    "#;
    assert_eq!(eval_int(code), 120);
}

#[test]
fn return_recursive_fibonacci() {
    let code = r#"
        fn fib(n) {
            if (n <= 1) {
                return n;
            } else {
                return fib(n - 1) + fib(n - 2);
            }
        }
        fib(10);
    "#;
    assert_eq!(eval_int(code), 55);
}

#[test]
fn return_mutual_recursion() {
    let code = r#"
        fn isEven(n) {
            if (n == 0) {
                return true;
            } else {
                return isOdd(n - 1);
            }
        }
        
        fn isOdd(n) {
            if (n == 0) {
                return false;
            } else {
                return isEven(n - 1);
            }
        }
        
        isEven(10);
    "#;
    assert_eq!(eval_bool(code), true);
}

// ============================================================================
// Function Without Return Tests
// ============================================================================

#[test]
fn function_without_return_yields_null() {
    let code = r#"
        fn doNothing() {
            let x = 1;
        }
        doNothing();
    "#;
    match eval(code) {
        Ok((Value::Literal(Literal::Null), _)) => {}
        other => panic!("Expected Null, got {:?}", other),
    }
}

// ============================================================================
// Return in Methods Tests
// ============================================================================

#[test]
fn return_from_method() {
    let code = r#"
        class Calculator {
            let value = 0;
            fn init(self, v) {
                self.value = v;
            }
            fn double(self) {
                return self.value * 2;
            }
        }
        let c = new Calculator(21);
        c.double();
    "#;
    assert_eq!(eval_int(code), 42);
}

#[test]
fn return_from_static_method() {
    let code = r#"
        class MathUtils {
            fn add(a, b) {
                return a + b;
            }
        }
        MathUtils::add(17, 25);
    "#;
    assert_eq!(eval_int(code), 42);
}

// ============================================================================
// Return Error Tests
// ============================================================================

#[test]
fn return_outside_function_is_error() {
    let code = r#"
        return 42;
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// Return with Complex Expressions
// ============================================================================

#[test]
fn return_computed_value() {
    let code = r#"
        fn sumSquares(n) {
            let total = 0;
            for i in range(1, n + 1) {
                total = total + i * i;
            }
            return total;
        }
        sumSquares(5);
    "#;
    // 1 + 4 + 9 + 16 + 25 = 55
    assert_eq!(eval_int(code), 55);
}

#[test]
fn return_function_call_result() {
    let code = r#"
        fn double(x) { return x * 2; }
        fn quadruple(x) { return double(double(x)); }
        quadruple(5);
    "#;
    assert_eq!(eval_int(code), 20);
}

#[test]
fn return_list() {
    let code = r#"
        fn makeList() {
            return [1, 2, 3];
        }
        makeList().len();
    "#;
    assert_eq!(eval_int(code), 3);
}
