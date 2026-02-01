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
// Division by Zero Tests
// ============================================================================

#[test]
fn error_division_by_zero_int() {
    let code = "10 / 0;";
    assert!(eval(code).is_err());
}

#[test]
fn error_division_by_zero_float() {
    let code = "10.0 / 0.0;";
    assert!(eval(code).is_err());
}

#[test]
fn error_division_by_zero_variable() {
    let code = r#"
        let x = 0;
        10 / x;
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// Undefined Variable Tests
// ============================================================================

#[test]
fn error_undefined_variable() {
    let code = "x;";
    assert!(eval(code).is_err());
}

#[test]
fn error_undefined_variable_in_expression() {
    let code = "let y = x + 1;";
    assert!(eval(code).is_err());
}

#[test]
fn error_assignment_to_undefined() {
    let code = "x = 10;";
    assert!(eval(code).is_err());
}

// ============================================================================
// Undefined Function Tests
// ============================================================================

#[test]
fn error_undefined_function() {
    let code = "nonexistent();";
    assert!(eval(code).is_err());
}

#[test]
fn error_undefined_function_with_args() {
    let code = "foo(1, 2, 3);";
    assert!(eval(code).is_err());
}

// ============================================================================
// Function Argument Mismatch Tests
// ============================================================================

#[test]
fn error_too_few_arguments() {
    let code = r#"
        fn add(a, b) {
            a + b;
        }
        add(1);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_too_many_arguments() {
    let code = r#"
        fn add(a, b) {
            a + b;
        }
        add(1, 2, 3);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_no_arguments_when_required() {
    let code = r#"
        fn greet(name) {
            name;
        }
        greet();
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// Invalid Variable Name Tests
// ============================================================================

#[test]
fn error_invalid_variable_name_starts_with_number() {
    let code = "let 123abc = 10;";
    // This should fail at parse time
    let mut parser = Parser::new(code);
    assert!(parser.parse().is_err());
}

// ============================================================================
// List Method Error Tests
// ============================================================================

#[test]
fn error_list_index_out_of_bounds() {
    let code = r#"
        let list = [1, 2, 3];
        list.get(10);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_list_len_with_arguments() {
    let code = r#"
        let list = [1, 2, 3];
        list.len(1);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_list_push_wrong_arg_count() {
    let code = r#"
        let list = [1, 2, 3];
        list.push(1, 2);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_list_get_wrong_arg_count() {
    let code = r#"
        let list = [1, 2, 3];
        list.get();
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_list_concat_wrong_type() {
    let code = r#"
        let list = [1, 2, 3];
        list.concat(42);
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// String Method Error Tests
// ============================================================================

#[test]
fn error_string_len_with_arguments() {
    let code = r#"
        let s = "hello";
        s.len(1);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_string_concat_wrong_type() {
    let code = r#"
        let s = "hello";
        s.concat(42);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_unknown_string_method() {
    let code = r#"
        let s = "hello";
        s.unknown();
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// Class Error Tests
// ============================================================================

#[test]
fn error_class_not_found() {
    let code = "let x = new NonExistent();";
    assert!(eval(code).is_err());
}

#[test]
fn error_method_not_found() {
    let code = r#"
        class Empty {
            fn init(self) {}
        }
        let e = new Empty();
        e.nonexistent();
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_property_not_found() {
    let code = r#"
        class Simple {
            let x = 1;
            fn init(self) {}
        }
        let s = new Simple();
        s.nonexistent;
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_parent_class_not_found() {
    let code = r#"
        class Child : NonExistent {
            let x = 1;
        }
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// Type Error Tests
// ============================================================================

#[test]
fn error_call_non_function() {
    let code = r#"
        let x = 42;
        x();
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn error_property_access_on_non_object() {
    let code = r#"
        let x = 42;
        x.property;
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// Edge Cases That Should Work
// ============================================================================

#[test]
fn edge_case_empty_block() {
    let code = "{}";
    assert!(eval(code).is_ok());
}

#[test]
fn edge_case_nested_empty_blocks() {
    let code = "{{{{}}}}";
    assert!(eval(code).is_ok());
}

#[test]
fn edge_case_empty_list() {
    let code = "let x = [];";
    assert!(eval(code).is_ok());
}

#[test]
fn edge_case_single_element_list() {
    let code = r#"
        let x = [42];
        x.get(0);
    "#;
    assert_eq!(eval_int(code), 42);
}

#[test]
fn edge_case_negative_index() {
    let code = r#"
        let x = [1, 2, 3];
        x.get(-1);
    "#;
    assert_eq!(eval_int(code), 3);
}

#[test]
fn edge_case_deeply_nested_expression() {
    let code = "(((((1 + 2) * 3) - 4) / 1) + 5);";
    // ((((3) * 3) - 4) / 1) + 5 = ((9 - 4) / 1) + 5 = (5 / 1) + 5 = 5 + 5 = 10
    assert_eq!(eval_int(code), 10);
}

#[test]
fn edge_case_many_nested_functions() {
    let code = r#"
        fn a(x) { return x + 1; }
        fn b(x) { return a(x) + 1; }
        fn c(x) { return b(x) + 1; }
        fn d(x) { return c(x) + 1; }
        fn e(x) { return d(x) + 1; }
        e(0);
    "#;
    assert_eq!(eval_int(code), 5);
}

#[test]
fn edge_case_recursive_fibonacci() {
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
fn edge_case_mutual_recursion() {
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
    match eval(code) {
        Ok(Value::Literal(Literal::Bool(b))) => assert!(b),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn edge_case_large_number() {
    let code = "let x = 9223372036854775807; x;"; // i64::MAX
    assert_eq!(eval_int(code), i64::MAX);
}

#[test]
fn edge_case_negative_number() {
    // Note: 9223372036854775808 overflows i64 so it's parsed as float
    // Use -9223372036854775807 (i64::MAX negated) instead
    let code = "let x = -9223372036854775807; x;";
    assert_eq!(eval_int(code), -9223372036854775807);
}

#[test]
fn edge_case_empty_string() {
    let code = r#"
        let s = "";
        s.len();
    "#;
    assert_eq!(eval_int(code), 0);
}

#[test]
fn edge_case_string_with_spaces() {
    let code = r#"
        let s = "hello world";
        s.len();
    "#;
    assert_eq!(eval_int(code), 11);
}

// ============================================================================
// Truthiness Edge Cases
// ============================================================================

#[test]
fn truthiness_zero_is_falsy() {
    let code = r#"
        fn test() {
            if (0) {
                return 1;
            } else {
                return 2;
            }
        }
        test();
    "#;
    assert_eq!(eval_int(code), 2);
}

#[test]
fn truthiness_null_is_falsy() {
    let code = r#"
        fn test(x) {
            if (x) {
                return 1;
            } else {
                return 2;
            }
        }
        test(null);
    "#;
    assert_eq!(eval_int(code), 2);
}

#[test]
fn truthiness_empty_string_is_truthy() {
    // Note: In Axe, empty string IS truthy according to the docs
    let code = r#"
        fn test() {
            if ("") {
                return 1;
            } else {
                return 2;
            }
        }
        test();
    "#;
    assert_eq!(eval_int(code), 1);
}

#[test]
fn truthiness_non_zero_is_truthy() {
    let code = r#"
        fn test() {
            if (42) {
                return 1;
            } else {
                return 2;
            }
        }
        test();
    "#;
    assert_eq!(eval_int(code), 1);
}

#[test]
fn truthiness_negative_is_truthy() {
    let code = r#"
        fn test() {
            if (-1) {
                return 1;
            } else {
                return 2;
            }
        }
        test();
    "#;
    assert_eq!(eval_int(code), 1);
}
