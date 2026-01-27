use axe::{Axe, Parser};

// =============================================================================
// Parser Tests - Testing that the Parser produces correct AST
// =============================================================================

#[test]
fn parse_integer_literal() {
    let mut parser = Parser::new("42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_negative_integer() {
    let mut parser = Parser::new("-42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_positive_integer() {
    let mut parser = Parser::new("+42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_zero() {
    let mut parser = Parser::new("0;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_large_integer() {
    let mut parser = Parser::new("9999999999;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_float_literal() {
    let mut parser = Parser::new("3.14;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_negative_float() {
    let mut parser = Parser::new("-3.14;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_float_with_trailing_zeros() {
    let mut parser = Parser::new("1.500;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_float_whole_number() {
    let mut parser = Parser::new("5.0;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// String Literal Tests
// =============================================================================

#[test]
fn parse_simple_string() {
    let mut parser = Parser::new("\"hello\";");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_empty_string() {
    let mut parser = Parser::new("\"\";");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_string_with_spaces() {
    let mut parser = Parser::new("\"hello world\";");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_string_with_numbers() {
    let mut parser = Parser::new("\"test123\";");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_string_with_special_chars() {
    let mut parser = Parser::new("\"!@#$%^&*()\";");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Error Cases Tests
// =============================================================================

#[test]
fn parse_missing_semicolon() {
    let mut parser = Parser::new("42");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn parse_unclosed_string() {
    let mut parser = Parser::new("\"hello;");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn parse_unclosed_block() {
    let mut parser = Parser::new("{ 42;");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn parse_extra_closing_brace() {
    let mut parser = Parser::new("42; }");
    let result = parser.parse();
    assert!(result.is_err());
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

#[test]
fn parse_very_long_string() {
    let long_string = "a".repeat(1000);
    let input = format!("\"{}\";", long_string);
    let mut parser = Parser::new(&input);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_unicode_string() {
    let mut parser = Parser::new("\"hello unicode\";");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Block Statement Tests
// =============================================================================

#[test]
fn parse_empty_block() {
    let mut parser = Parser::new("{}");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_block_with_single_statement() {
    let mut parser = Parser::new("{ 42; }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_block_with_string() {
    let mut parser = Parser::new("{ \"hello\"; }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_block_with_formatting() {
    let mut parser = Parser::new("{\n    42;\n}");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Whitespace Tests
// =============================================================================

#[test]
fn parse_with_leading_whitespace() {
    let mut parser = Parser::new("   42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_with_trailing_whitespace() {
    let mut parser = Parser::new("42;   ");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_with_newlines() {
    let mut parser = Parser::new("\n42;\n");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_with_tabs() {
    let mut parser = Parser::new("\t42;\t");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Comment Tests
// =============================================================================

#[test]
fn parse_with_line_comment() {
    let mut parser = Parser::new("// this is a comment\n42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_with_inline_comment() {
    let mut parser = Parser::new("42; // inline comment");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_with_block_comment() {
    let mut parser = Parser::new("/* block comment */ 42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_with_multiline_block_comment() {
    let mut parser = Parser::new("/*\n  multiline\n  comment\n*/ 42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Multiple Statements Tests
// =============================================================================

#[test]
fn parse_two_statements() {
    let mut parser = Parser::new("42; 100;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_three_statements() {
    let mut parser = Parser::new("1; 2; 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_statements_on_multiple_lines() {
    let mut parser = Parser::new("42;\n100;\n200;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_statements_with_blank_lines() {
    let mut parser = Parser::new("42;\n\n100;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_nested_empty_blocks() {
    let mut parser = Parser::new("{ {} }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_nested_block_with_statement() {
    let mut parser = Parser::new("{ { 42; } }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_deeply_nested_blocks() {
    let mut parser = Parser::new("{ { { 42; } } }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_sibling_blocks() {
    let mut parser = Parser::new("{ { 1; } { 2; } }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_block_with_multiple_statements() {
    let mut parser = Parser::new("{ 1; 2; 3; }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_nested_block_with_multiple_statements() {
    let mut parser = Parser::new("{ 1; { 2; 3; } 4; }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_complex_nested_structure() {
    let input = r#"{
        1;
        {
            2;
            { 3; }
        }
        4;
    }"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Addition Expression Tests
// =============================================================================

#[test]
fn parse_simple_addition() {
    let mut parser = Parser::new("1 + 2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_chained_addition() {
    let mut parser = Parser::new("1 + 2 + 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_parenthesized_addition() {
    let mut parser = Parser::new("(1 + 2) + 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_right_grouped_addition() {
    let mut parser = Parser::new("1 + (2 + 3);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_addition_no_spaces() {
    let mut parser = Parser::new("1+2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_addition_with_floats() {
    let mut parser = Parser::new("1.5 + 2.5;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_nested_parentheses() {
    let mut parser = Parser::new("((1 + 2));");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Subtraction Expression Tests
// =============================================================================

#[test]
fn parse_simple_subtraction() {
    let mut parser = Parser::new("5 - 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_chained_subtraction() {
    let mut parser = Parser::new("10 - 3 - 2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_subtraction_with_parentheses() {
    let mut parser = Parser::new("10 - (3 - 2);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_mixed_addition_subtraction() {
    let mut parser = Parser::new("1 + 2 - 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_subtraction_no_spaces() {
    let mut parser = Parser::new("5-3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Multiplication Expression Tests
// =============================================================================

#[test]
fn parse_simple_multiplication() {
    let mut parser = Parser::new("3 * 4;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_chained_multiplication() {
    let mut parser = Parser::new("2 * 3 * 4;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_multiplication_with_parentheses() {
    let mut parser = Parser::new("2 * (3 * 4);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_multiplication_no_spaces() {
    let mut parser = Parser::new("3*4;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_multiplication_with_floats() {
    let mut parser = Parser::new("2.5 * 4.0;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Division Expression Tests
// =============================================================================

#[test]
fn parse_simple_division() {
    let mut parser = Parser::new("10 / 2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_chained_division() {
    let mut parser = Parser::new("100 / 10 / 2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_division_with_parentheses() {
    let mut parser = Parser::new("100 / (10 / 2);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_mixed_multiplication_division() {
    let mut parser = Parser::new("2 * 6 / 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_division_no_spaces() {
    let mut parser = Parser::new("10/2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Operator Precedence Tests
// =============================================================================

#[test]
fn parse_precedence_mul_over_add() {
    let mut parser = Parser::new("1 + 2 * 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_precedence_div_over_sub() {
    let mut parser = Parser::new("10 - 6 / 2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_precedence_mul_before_sub() {
    let mut parser = Parser::new("2 * 3 - 1;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_precedence_override_with_parentheses() {
    let mut parser = Parser::new("(1 + 2) * 3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_complex_precedence() {
    let mut parser = Parser::new("1 + 2 * 3 - 4 / 2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_all_operators_chained() {
    let mut parser = Parser::new("10 + 5 - 3 * 2 / 1;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Unary Operator Tests
// =============================================================================

#[test]
fn parse_unary_minus_in_expression() {
    let mut parser = Parser::new("5 + -3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_unary_plus_in_expression() {
    let mut parser = Parser::new("5 + +3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_double_unary_minus_with_parens() {
    let mut parser = Parser::new("-(-5);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_unary_minus_with_parentheses() {
    let mut parser = Parser::new("-(1 + 2);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_unary_in_multiplicative() {
    let mut parser = Parser::new("2 * -3;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Assignment Tests
// =============================================================================

#[test]
fn parse_simple_assignment() {
    let mut parser = Parser::new("x = 42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_let_declaration() {
    let mut parser = Parser::new("let x = 42;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_let_with_expression() {
    let mut parser = Parser::new("let x = 1 + 2;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Boolean Literal Tests
// =============================================================================

#[test]
fn parse_true_literal() {
    let mut parser = Parser::new("true;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_false_literal() {
    let mut parser = Parser::new("false;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_boolean_in_let() {
    let mut parser = Parser::new("let x = true;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_boolean_in_assignment() {
    let mut parser = Parser::new("x = false;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Null Literal Tests
// =============================================================================

#[test]
fn parse_null_literal() {
    let mut parser = Parser::new("null;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_null_in_let() {
    let mut parser = Parser::new("let x = null;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_null_in_assignment() {
    let mut parser = Parser::new("x = null;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_let_without_initializer_is_null() {
    let mut parser = Parser::new("let x;");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// If conditional Tests
// =============================================================================

#[test]
fn parse_simple_if_statement() {
    let mut parser = Parser::new("let y; let x = 10; if (x > 1) { y = 42; } else { y = 0; }");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Evaluation Tests (end-to-end)
// =============================================================================

#[test]
fn eval_simple_addition() {
    let mut parser = Parser::new("1 + 2;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_chained_addition() {
    let mut parser = Parser::new("1 + 2 + 3;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_parenthesized_addition() {
    let mut parser = Parser::new("(10 + 20) + 30;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_float_addition() {
    let mut parser = Parser::new("1.5 + 2.5;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_simple_subtraction() {
    let mut parser = Parser::new("10 - 3;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_simple_multiplication() {
    let mut parser = Parser::new("3 * 4;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_simple_division() {
    let mut parser = Parser::new("10 / 2;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_precedence_mul_over_add() {
    let mut parser = Parser::new("1 + 2 * 3;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_precedence_with_parentheses() {
    let mut parser = Parser::new("(1 + 2) * 3;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_complex_expression() {
    let mut parser = Parser::new("2 + 3 * 4 - 10 / 2;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_unary_minus_in_expression() {
    let mut parser = Parser::new("5 + -3;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_unary_minus_with_multiplication() {
    let mut parser = Parser::new("2 * -3;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_nested_parentheses_complex() {
    let mut parser = Parser::new("((2 + 3) * (4 - 1));");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_true_literal() {
    let mut parser = Parser::new("true;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_false_literal() {
    let mut parser = Parser::new("false;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_null_literal() {
    let mut parser = Parser::new("null;");
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn parse_while_count_1_to_10_sum() {
    // Use a while loop to count from 1 to 10 and calculate sum
    let code = r#"
        let i = 1;
        let sum = 0;
        where (i <= 10) {
            sum = sum + i;
            i = i + 1;
        }
        sum == 55;
    "#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn eval_while_count_1_to_10_sum_is_correct() {
    // Use a while loop to count from 1 to 10 and calculate sum
    // Expected: 1+2+3+4+5+6+7+8+9+10 = 55
    let code = r#"
        let i = 1;
        let sum = 0;
        where (i <= 10) {
            sum = sum + i;
            i = i + 1;
        }
        sum == 55;
    "#;
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program).unwrap();
    // sum == 55 should evaluate to true
    assert!(matches!(
        result,
        axe::Value::Literal(axe::Literal::Bool(true))
    ));
}

// =============================================================================
// Function Call Tests
// =============================================================================

#[test]
fn parse_function_call_no_args() {
    let mut parser = Parser::new("foo();");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_single_arg() {
    let mut parser = Parser::new("print(42);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_multiple_args() {
    let mut parser = Parser::new("add(1, 2, 3);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_string_arg() {
    let mut parser = Parser::new("print(\"hello\");");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_expression_arg() {
    let mut parser = Parser::new("print(1 + 2);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_variable_arg() {
    let mut parser = Parser::new("print(x);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_mixed_args() {
    let mut parser = Parser::new("foo(1, \"hello\", x, 2 + 3);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_nested_function_call() {
    let mut parser = Parser::new("print(add(1, 2));");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_in_expression() {
    let mut parser = Parser::new("1 + foo(2);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_in_let() {
    let mut parser = Parser::new("let x = foo(42);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_in_assignment() {
    let mut parser = Parser::new("x = bar(1, 2);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_boolean_arg() {
    let mut parser = Parser::new("check(true);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_null_arg() {
    let mut parser = Parser::new("reset(null);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_multiple_function_calls() {
    let mut parser = Parser::new("foo(); bar(); baz();");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_whitespace() {
    let mut parser = Parser::new("foo( 1 , 2 );");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_function_call_multiline_args() {
    let mut parser = Parser::new("foo(\n    1,\n    2,\n    3\n);");
    let result = parser.parse();
    assert!(result.is_ok());
}

// =============================================================================
// Method Call Tests
// =============================================================================

#[test]
fn parse_method_call_no_args() {
    let mut parser = Parser::new("foo.bar();");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_method_call_with_args() {
    let mut parser = Parser::new("foo.bar(1, 2);");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_chained_method_calls() {
    let mut parser = Parser::new("foo.bar().baz();");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_method_call_on_string() {
    let mut parser = Parser::new(r#""hello".len();"#);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_method_call_after_function() {
    let mut parser = Parser::new("getUser().name();");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_property_access() {
    let mut parser = Parser::new("foo.bar;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_chained_property_access() {
    let mut parser = Parser::new("foo.bar.baz;");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn parse_property_then_method() {
    let mut parser = Parser::new("foo.bar.len();");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn eval_string_len_method() {
    let code = r#""hello".len();"#;
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(5))));
}

#[test]
fn eval_string_concat_method() {
    let code = r#""hello".concat(" world");"#;
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Str(_))));
}

#[test]
fn eval_method_on_variable() {
    let code = r#"
        let s = "hello";
        s.len();
    "#;
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(5))));
}

#[test]
fn eval_chained_method_calls() {
    let code = r#""a".concat("b").concat("c");"#;
    let mut parser = Parser::new(code);
    let program = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.run(program);
    assert!(result.is_ok());
}
