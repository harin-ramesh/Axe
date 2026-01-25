use axe::{Axe, Literal, Parser, Value};

fn eval(input: &str) -> Value {
    let mut parser = Parser::new(input);
    let program = parser.parse().expect("parse failed");
    let axe = Axe::new();
    axe.run(program).expect("eval failed")
}

fn eval_int(input: &str) -> i64 {
    match eval(input) {
        Value::Literal(Literal::Int(n)) => n,
        other => panic!("Expected Int, got {:?}", other),
    }
}

fn eval_float(input: &str) -> f64 {
    match eval(input) {
        Value::Literal(Literal::Float(f)) => f,
        other => panic!("Expected Float, got {:?}", other),
    }
}

// =============================================================================
// Unary Minus Tests - Integers
// =============================================================================

#[test]
fn unary_minus_integer() {
    assert_eq!(eval_int("-5;"), -5);
}

#[test]
fn unary_minus_zero() {
    assert_eq!(eval_int("-0;"), 0);
}

#[test]
fn unary_minus_in_addition() {
    assert_eq!(eval_int("5 + -3;"), 2);
}

#[test]
fn unary_minus_in_subtraction() {
    assert_eq!(eval_int("5 - -3;"), 8);
}

#[test]
fn unary_minus_in_multiplication() {
    assert_eq!(eval_int("2 * -3;"), -6);
}

#[test]
fn unary_minus_in_division() {
    assert_eq!(eval_int("10 / -2;"), -5);
}

#[test]
fn unary_minus_in_modulo() {
    assert_eq!(eval_int("10 % -3;"), 1);
}

#[test]
fn unary_minus_with_parentheses() {
    assert_eq!(eval_int("-(1 + 2);"), -3);
}

#[test]
fn unary_minus_complex_expression() {
    assert_eq!(eval_int("-(2 * 3 + 4);"), -10);
}

#[test]
fn double_unary_minus() {
    assert_eq!(eval_int("-(-5);"), 5);
}

#[test]
fn triple_unary_minus() {
    assert_eq!(eval_int("-(-(-5));"), -5);
}

#[test]
fn unary_minus_with_variable() {
    assert_eq!(eval_int("let x = 10; -x;"), -10);
}

#[test]
fn unary_minus_in_variable_assignment() {
    assert_eq!(eval_int("let x = -5; x;"), -5);
}

#[test]
fn unary_minus_both_operands() {
    assert_eq!(eval_int("-5 + -3;"), -8);
}

#[test]
fn unary_minus_precedence_with_multiplication() {
    // -2 * 3 should be (-2) * 3 = -6, not -(2 * 3)
    assert_eq!(eval_int("-2 * 3;"), -6);
}

#[test]
fn unary_minus_right_side_of_multiplication() {
    assert_eq!(eval_int("3 * -2;"), -6);
}

#[test]
fn unary_minus_chained_operations() {
    assert_eq!(eval_int("-1 + -2 + -3;"), -6);
}

#[test]
fn unary_minus_nested_parentheses() {
    assert_eq!(eval_int("-(-(-(1 + 2)));"), -3);
}

// =============================================================================
// Unary Minus Tests - Floats
// =============================================================================

#[test]
fn unary_minus_float() {
    assert_eq!(eval_float("-3.14;"), -3.14);
}

#[test]
fn unary_minus_float_zero() {
    assert_eq!(eval_float("-0.0;"), 0.0);
}

#[test]
fn unary_minus_float_in_expression() {
    assert_eq!(eval_float("5.0 + -3.0;"), 2.0);
}

#[test]
fn double_unary_minus_float() {
    assert_eq!(eval_float("-(-3.14);"), 3.14);
}

// =============================================================================
// Unary Plus Tests
// =============================================================================

#[test]
fn unary_plus_integer() {
    assert_eq!(eval_int("+5;"), 5);
}

#[test]
fn unary_plus_negative_becomes_positive() {
    // +(-5) is still -5, unary plus doesn't change sign
    assert_eq!(eval_int("+(-5);"), -5);
}

#[test]
fn unary_plus_in_expression() {
    assert_eq!(eval_int("5 + +3;"), 8);
}

#[test]
fn unary_plus_with_variable() {
    assert_eq!(eval_int("let x = 10; +x;"), 10);
}

#[test]
fn unary_plus_no_effect() {
    assert_eq!(eval_int("+(+5);"), 5);
}

// =============================================================================
// Mixed Unary Operators
// =============================================================================

#[test]
fn unary_plus_and_minus_mixed() {
    assert_eq!(eval_int("+5 + -3;"), 2);
}

#[test]
fn unary_minus_in_if_condition() {
    assert_eq!(eval_int("let x = -1; if (x < 0) { 1; } else { 0; }"), 1);
}

// Note: Comparison operators (< > <= >= == !=) are only supported inside
// if/while conditions, not as standalone expressions. See parser.rs parse_condition().

#[test]
fn unary_minus_in_if_less_than() {
    // Use if statement to test comparison with negative numbers
    assert_eq!(eval_int("if ((-5) < 0) { 1; } else { 0; }"), 1);
}

#[test]
fn unary_minus_in_if_greater_than() {
    assert_eq!(eval_int("if (0 > (-5)) { 1; } else { 0; }"), 1);
}

#[test]
fn unary_minus_in_if_equality() {
    assert_eq!(eval_int("if ((-5) == (-5)) { 1; } else { 0; }"), 1);
}

#[test]
fn unary_minus_in_if_inequality() {
    assert_eq!(eval_int("if ((-5) != 5) { 1; } else { 0; }"), 1);
}

#[test]
fn unary_minus_in_if_less_than_or_equal() {
    assert_eq!(eval_int("if ((-5) <= (-5)) { 1; } else { 0; }"), 1);
}

#[test]
fn unary_minus_in_if_greater_than_or_equal() {
    assert_eq!(eval_int("if ((-5) >= (-10)) { 1; } else { 0; }"), 1);
}

// =============================================================================
// Logical Not Tests (!)
// =============================================================================

fn eval_bool(input: &str) -> bool {
    match eval(input) {
        Value::Literal(Literal::Bool(b)) => b,
        other => panic!("Expected Bool, got {:?}", other),
    }
}

#[test]
fn logical_not_true() {
    assert_eq!(eval_bool("!true;"), false);
}

#[test]
fn logical_not_false() {
    assert_eq!(eval_bool("!false;"), true);
}

#[test]
fn logical_not_double() {
    assert_eq!(eval_bool("!!true;"), true);
}

#[test]
fn logical_not_triple() {
    assert_eq!(eval_bool("!!!true;"), false);
}

#[test]
fn logical_not_zero_is_true() {
    // 0 is falsy, so !0 is true
    assert_eq!(eval_bool("!0;"), true);
}

#[test]
fn logical_not_nonzero_is_false() {
    // non-zero is truthy, so !1 is false
    assert_eq!(eval_bool("!1;"), false);
    assert_eq!(eval_bool("!42;"), false);
    assert_eq!(eval_bool("!(-1);"), false);
}

#[test]
fn logical_not_null_is_true() {
    // null is falsy
    assert_eq!(eval_bool("!null;"), true);
}

#[test]
fn logical_not_empty_string_is_true() {
    // empty string is falsy
    assert_eq!(eval_bool("!\"\";"), true);
}

#[test]
fn logical_not_nonempty_string_is_false() {
    // non-empty string is truthy
    assert_eq!(eval_bool("!\"hello\";"), false);
}

#[test]
fn logical_not_float_zero_is_true() {
    assert_eq!(eval_bool("!0.0;"), true);
}

#[test]
fn logical_not_float_nonzero_is_false() {
    assert_eq!(eval_bool("!3.14;"), false);
}

#[test]
fn logical_not_with_variable() {
    assert_eq!(eval_bool("let x = true; !x;"), false);
}

#[test]
fn logical_not_in_if_condition() {
    assert_eq!(eval_int("if (!false) { 1; } else { 0; }"), 1);
}

#[test]
fn logical_not_combined_with_and() {
    assert_eq!(eval_bool("!false && true;"), true);
}

#[test]
fn logical_not_combined_with_or() {
    assert_eq!(eval_bool("!true || true;"), true);
}

// =============================================================================
// Bitwise Invert Tests (~)
// =============================================================================

#[test]
fn bitwise_invert_zero() {
    assert_eq!(eval_int("~0;"), -1);
}

#[test]
fn bitwise_invert_one() {
    assert_eq!(eval_int("~1;"), -2);
}

#[test]
fn bitwise_invert_negative_one() {
    assert_eq!(eval_int("~(-1);"), 0);
}

#[test]
fn bitwise_invert_positive() {
    // ~5 in two's complement: 5 = 0...0101, ~5 = 1...1010 = -6
    assert_eq!(eval_int("~5;"), -6);
}

#[test]
fn bitwise_invert_double() {
    // ~~x == x
    assert_eq!(eval_int("~~42;"), 42);
}

#[test]
fn bitwise_invert_with_variable() {
    assert_eq!(eval_int("let x = 10; ~x;"), -11);
}

#[test]
fn bitwise_invert_in_expression() {
    assert_eq!(eval_int("~0 + 1;"), 0);
}

#[test]
fn bitwise_invert_combined_with_and() {
    // ~0 = -1 (all 1s), -1 & 5 = 5
    assert_eq!(eval_int("~0 & 5;"), 5);
}

#[test]
fn bitwise_invert_combined_with_or() {
    // ~(-1) = 0, 0 | 5 = 5
    assert_eq!(eval_int("~(-1) | 5;"), 5);
}

// =============================================================================
// Mixed Unary Operators
// =============================================================================

#[test]
fn mixed_unary_neg_and_not() {
    // -(!true) doesn't make sense, but !(-5) does
    assert_eq!(eval_bool("!(-5);"), false); // -5 is truthy
}

#[test]
fn mixed_unary_neg_and_inv() {
    // ~(-1) = 0, then negate: -0 = 0
    assert_eq!(eval_int("-~(-1);"), 0);
}

#[test]
fn mixed_unary_not_and_inv() {
    // ~0 = -1 (truthy), !(-1) = false
    assert_eq!(eval_bool("!~0;"), false);
}
