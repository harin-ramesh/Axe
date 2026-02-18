use axe::{Axe, Context, Parser};

// =============================================================================
// Parser Tests - Testing that the Parser produces correct AST
// =============================================================================

/// Helper function to parse source code with a fresh context.
fn parse(source: &str) -> Result<axe::Program, &'static str> {
    let ctx = Context::new();
    let mut parser = Parser::new(source, &ctx);
    parser.parse()
}

fn run_code(source: &str) -> Result<axe::Value, axe::EvalSignal> {
    let ctx = Context::new();
    let mut parser = Parser::new(source, &ctx);
    let program = parser
        .parse()
        .map_err(|e| axe::EvalSignal::Error(e.to_string()))?;
    let mut axe = Axe::new(&ctx);
    axe.run(program)
}

#[test]
fn parse_integer_literal() {
    let result = parse("42;");
    assert!(result.is_ok());
}

#[test]
fn parse_negative_integer() {
    let result = parse("-42;");

    assert!(result.is_ok());
}

#[test]
fn parse_positive_integer() {
    let result = parse("+42;");

    assert!(result.is_ok());
}

#[test]
fn parse_zero() {
    let result = parse("0;");

    assert!(result.is_ok());
}

#[test]
fn parse_large_integer() {
    let result = parse("9999999999;");

    assert!(result.is_ok());
}

#[test]
fn parse_float_literal() {
    let result = parse("3.14;");

    assert!(result.is_ok());
}

#[test]
fn parse_negative_float() {
    let result = parse("-3.14;");

    assert!(result.is_ok());
}

#[test]
fn parse_float_with_trailing_zeros() {
    let result = parse("1.500;");

    assert!(result.is_ok());
}

#[test]
fn parse_float_whole_number() {
    let result = parse("5.0;");

    assert!(result.is_ok());
}

// =============================================================================
// String Literal Tests
// =============================================================================

#[test]
fn parse_simple_string() {
    let result = parse("\"hello\";");

    assert!(result.is_ok());
}

#[test]
fn parse_empty_string() {
    let result = parse("\"\";");

    assert!(result.is_ok());
}

#[test]
fn parse_string_with_spaces() {
    let result = parse("\"hello world\";");

    assert!(result.is_ok());
}

#[test]
fn parse_string_with_numbers() {
    let result = parse("\"test123\";");

    assert!(result.is_ok());
}

#[test]
fn parse_string_with_special_chars() {
    let result = parse("\"!@#$%^&*()\";");

    assert!(result.is_ok());
}

// =============================================================================
// Error Cases Tests
// =============================================================================

#[test]
fn parse_missing_semicolon() {
    let result = parse("42");

    assert!(result.is_err());
}

#[test]
fn parse_unclosed_string() {
    let result = parse("\"hello;");

    assert!(result.is_err());
}

#[test]
fn parse_unclosed_block() {
    let result = parse("{ 42;");

    assert!(result.is_err());
}

#[test]
fn parse_extra_closing_brace() {
    let result = parse("42; }");

    assert!(result.is_err());
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

#[test]
fn parse_very_long_string() {
    let long_string = "a".repeat(1000);
    let input = format!("\"{}\";", long_string);
    let result = parse(&input);

    assert!(result.is_ok());
}

#[test]
fn parse_unicode_string() {
    let result = parse("\"hello unicode\";");

    assert!(result.is_ok());
}

// =============================================================================
// Block Statement Tests
// =============================================================================

#[test]
fn parse_empty_block() {
    let result = parse("{}");

    assert!(result.is_ok());
}

#[test]
fn parse_block_with_single_statement() {
    let result = parse("{ 42; }");

    assert!(result.is_ok());
}

#[test]
fn parse_block_with_string() {
    let result = parse("{ \"hello\"; }");

    assert!(result.is_ok());
}

#[test]
fn parse_block_with_formatting() {
    let result = parse("{\n    42;\n}");

    assert!(result.is_ok());
}

// =============================================================================
// Whitespace Tests
// =============================================================================

#[test]
fn parse_with_leading_whitespace() {
    let result = parse("   42;");

    assert!(result.is_ok());
}

#[test]
fn parse_with_trailing_whitespace() {
    let result = parse("42;   ");

    assert!(result.is_ok());
}

#[test]
fn parse_with_newlines() {
    let result = parse("\n42;\n");

    assert!(result.is_ok());
}

#[test]
fn parse_with_tabs() {
    let result = parse("\t42;\t");

    assert!(result.is_ok());
}

// =============================================================================
// Comment Tests
// =============================================================================

#[test]
fn parse_with_line_comment() {
    let result = parse("// this is a comment\n42;");

    assert!(result.is_ok());
}

#[test]
fn parse_with_inline_comment() {
    let result = parse("42; // inline comment");

    assert!(result.is_ok());
}

#[test]
fn parse_with_block_comment() {
    let result = parse("/* block comment */ 42;");

    assert!(result.is_ok());
}

#[test]
fn parse_with_multiline_block_comment() {
    let result = parse("/*\n  multiline\n  comment\n*/ 42;");

    assert!(result.is_ok());
}

// =============================================================================
// Multiple Statements Tests
// =============================================================================

#[test]
fn parse_two_statements() {
    let result = parse("42; 100;");

    assert!(result.is_ok());
}

#[test]
fn parse_three_statements() {
    let result = parse("1; 2; 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_statements_on_multiple_lines() {
    let result = parse("42;\n100;\n200;");

    assert!(result.is_ok());
}

#[test]
fn parse_statements_with_blank_lines() {
    let result = parse("42;\n\n100;");

    assert!(result.is_ok());
}

#[test]
fn parse_nested_empty_blocks() {
    let result = parse("{ {} }");

    assert!(result.is_ok());
}

#[test]
fn parse_nested_block_with_statement() {
    let result = parse("{ { 42; } }");

    assert!(result.is_ok());
}

#[test]
fn parse_deeply_nested_blocks() {
    let result = parse("{ { { 42; } } }");

    assert!(result.is_ok());
}

#[test]
fn parse_sibling_blocks() {
    let result = parse("{ { 1; } { 2; } }");

    assert!(result.is_ok());
}

#[test]
fn parse_block_with_multiple_statements() {
    let result = parse("{ 1; 2; 3; }");

    assert!(result.is_ok());
}

#[test]
fn parse_nested_block_with_multiple_statements() {
    let result = parse("{ 1; { 2; 3; } 4; }");

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
    let result = parse(input);

    assert!(result.is_ok());
}

// =============================================================================
// Addition Expression Tests
// =============================================================================

#[test]
fn parse_simple_addition() {
    let result = parse("1 + 2;");

    assert!(result.is_ok());
}

#[test]
fn parse_chained_addition() {
    let result = parse("1 + 2 + 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_parenthesized_addition() {
    let result = parse("(1 + 2) + 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_right_grouped_addition() {
    let result = parse("1 + (2 + 3);");

    assert!(result.is_ok());
}

#[test]
fn parse_addition_no_spaces() {
    let result = parse("1+2;");

    assert!(result.is_ok());
}

#[test]
fn parse_addition_with_floats() {
    let result = parse("1.5 + 2.5;");

    assert!(result.is_ok());
}

#[test]
fn parse_nested_parentheses() {
    let result = parse("((1 + 2));");

    assert!(result.is_ok());
}

// =============================================================================
// Subtraction Expression Tests
// =============================================================================

#[test]
fn parse_simple_subtraction() {
    let result = parse("5 - 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_chained_subtraction() {
    let result = parse("10 - 3 - 2;");

    assert!(result.is_ok());
}

#[test]
fn parse_subtraction_with_parentheses() {
    let result = parse("10 - (3 - 2);");

    assert!(result.is_ok());
}

#[test]
fn parse_mixed_addition_subtraction() {
    let result = parse("1 + 2 - 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_subtraction_no_spaces() {
    let result = parse("5-3;");

    assert!(result.is_ok());
}

// =============================================================================
// Multiplication Expression Tests
// =============================================================================

#[test]
fn parse_simple_multiplication() {
    let result = parse("3 * 4;");

    assert!(result.is_ok());
}

#[test]
fn parse_chained_multiplication() {
    let result = parse("2 * 3 * 4;");

    assert!(result.is_ok());
}

#[test]
fn parse_multiplication_with_parentheses() {
    let result = parse("2 * (3 * 4);");

    assert!(result.is_ok());
}

#[test]
fn parse_multiplication_no_spaces() {
    let result = parse("3*4;");

    assert!(result.is_ok());
}

#[test]
fn parse_multiplication_with_floats() {
    let result = parse("2.5 * 4.0;");

    assert!(result.is_ok());
}

// =============================================================================
// Division Expression Tests
// =============================================================================

#[test]
fn parse_simple_division() {
    let result = parse("10 / 2;");

    assert!(result.is_ok());
}

#[test]
fn parse_chained_division() {
    let result = parse("100 / 10 / 2;");

    assert!(result.is_ok());
}

#[test]
fn parse_division_with_parentheses() {
    let result = parse("100 / (10 / 2);");

    assert!(result.is_ok());
}

#[test]
fn parse_mixed_multiplication_division() {
    let result = parse("2 * 6 / 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_division_no_spaces() {
    let result = parse("10/2;");

    assert!(result.is_ok());
}

// =============================================================================
// Operator Precedence Tests
// =============================================================================

#[test]
fn parse_precedence_mul_over_add() {
    let result = parse("1 + 2 * 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_precedence_div_over_sub() {
    let result = parse("10 - 6 / 2;");

    assert!(result.is_ok());
}

#[test]
fn parse_precedence_mul_before_sub() {
    let result = parse("2 * 3 - 1;");

    assert!(result.is_ok());
}

#[test]
fn parse_precedence_override_with_parentheses() {
    let result = parse("(1 + 2) * 3;");

    assert!(result.is_ok());
}

#[test]
fn parse_complex_precedence() {
    let result = parse("1 + 2 * 3 - 4 / 2;");

    assert!(result.is_ok());
}

#[test]
fn parse_all_operators_chained() {
    let result = parse("10 + 5 - 3 * 2 / 1;");

    assert!(result.is_ok());
}

// =============================================================================
// Unary Operator Tests
// =============================================================================

#[test]
fn parse_unary_minus_in_expression() {
    let result = parse("5 + -3;");

    assert!(result.is_ok());
}

#[test]
fn parse_unary_plus_in_expression() {
    let result = parse("5 + +3;");

    assert!(result.is_ok());
}

#[test]
fn parse_double_unary_minus_with_parens() {
    let result = parse("-(-5);");

    assert!(result.is_ok());
}

#[test]
fn parse_unary_minus_with_parentheses() {
    let result = parse("-(1 + 2);");

    assert!(result.is_ok());
}

#[test]
fn parse_unary_in_multiplicative() {
    let result = parse("2 * -3;");

    assert!(result.is_ok());
}

// =============================================================================
// Assignment Tests
// =============================================================================

#[test]
fn parse_simple_assignment() {
    let result = parse("x = 42;");

    assert!(result.is_ok());
}

#[test]
fn parse_let_declaration() {
    let result = parse("let x = 42;");

    assert!(result.is_ok());
}

#[test]
fn parse_let_with_expression() {
    let result = parse("let x = 1 + 2;");

    assert!(result.is_ok());
}

// =============================================================================
// Boolean Literal Tests
// =============================================================================

#[test]
fn parse_true_literal() {
    let result = parse("true;");

    assert!(result.is_ok());
}

#[test]
fn parse_false_literal() {
    let result = parse("false;");

    assert!(result.is_ok());
}

#[test]
fn parse_boolean_in_let() {
    let result = parse("let x = true;");

    assert!(result.is_ok());
}

#[test]
fn parse_boolean_in_assignment() {
    let result = parse("x = false;");

    assert!(result.is_ok());
}

// =============================================================================
// Null Literal Tests
// =============================================================================

#[test]
fn parse_null_literal() {
    let result = parse("null;");

    assert!(result.is_ok());
}

#[test]
fn parse_null_in_let() {
    let result = parse("let x = null;");

    assert!(result.is_ok());
}

#[test]
fn parse_null_in_assignment() {
    let result = parse("x = null;");

    assert!(result.is_ok());
}

#[test]
fn parse_let_without_initializer_is_null() {
    let result = parse("let x;");

    assert!(result.is_ok());
}

// =============================================================================
// If conditional Tests
// =============================================================================

#[test]
fn parse_simple_if_statement() {
    let result = parse("let y; let x = 10; if (x > 1) { y = 42; } else { y = 0; }");

    assert!(result.is_ok());
}

// =============================================================================
// Evaluation Tests (end-to-end)
// =============================================================================

#[test]
fn eval_simple_addition() {
    let program = parse("1 + 2;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_chained_addition() {
    let program = parse("1 + 2 + 3;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_parenthesized_addition() {
    let program = parse("(10 + 20) + 30;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_float_addition() {
    let program = parse("1.5 + 2.5;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_simple_subtraction() {
    let program = parse("10 - 3;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_simple_multiplication() {
    let program = parse("3 * 4;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_simple_division() {
    let program = parse("10 / 2;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_precedence_mul_over_add() {
    let program = parse("1 + 2 * 3;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_precedence_with_parentheses() {
    let program = parse("(1 + 2) * 3;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_complex_expression() {
    let program = parse("2 + 3 * 4 - 10 / 2;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_unary_minus_in_expression() {
    let program = parse("5 + -3;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_unary_minus_with_multiplication() {
    let program = parse("2 * -3;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_nested_parentheses_complex() {
    let program = parse("((2 + 3) * (4 - 1));").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_true_literal() {
    let program = parse("true;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_false_literal() {
    let program = parse("false;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn eval_null_literal() {
    let program = parse("null;").unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program);
    assert!(result.is_ok());
}

#[test]
fn parse_while_count_1_to_10_sum() {
    // Use a while loop to count from 1 to 10 and calculate sum
    let code = r#"
        let i = 1;
        let sum = 0;
        while (i <= 10) {
            sum = sum + i;
            i = i + 1;
        }
        sum == 55;
    "#;
    let result = parse(code);

    assert!(result.is_ok());
}

#[test]
fn eval_while_count_1_to_10_sum_is_correct() {
    // Use a while loop to count from 1 to 10 and calculate sum
    // Expected: 1+2+3+4+5+6+7+8+9+10 = 55
    let code = r#"
        let i = 1;
        let sum = 0;
        while (i <= 10) {
            sum = sum + i;
            i = i + 1;
        }
        sum == 55;
    "#;
    let program = parse(code).unwrap();

    let context = Context::new();
    let mut axe = Axe::new(&context);
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
    let result = parse("foo();");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_single_arg() {
    let result = parse("print(42);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_multiple_args() {
    let result = parse("add(1, 2, 3);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_string_arg() {
    let result = parse("print(\"hello\");");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_expression_arg() {
    let result = parse("print(1 + 2);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_variable_arg() {
    let result = parse("print(x);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_mixed_args() {
    let result = parse("foo(1, \"hello\", x, 2 + 3);");

    assert!(result.is_ok());
}

#[test]
fn parse_nested_function_call() {
    let result = parse("print(add(1, 2));");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_in_expression() {
    let result = parse("1 + foo(2);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_in_let() {
    let result = parse("let x = foo(42);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_in_assignment() {
    let result = parse("x = bar(1, 2);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_boolean_arg() {
    let result = parse("check(true);");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_null_arg() {
    let result = parse("reset(null);");

    assert!(result.is_ok());
}

#[test]
fn parse_multiple_function_calls() {
    let result = parse("foo(); bar(); baz();");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_with_whitespace() {
    let result = parse("foo( 1 , 2 );");

    assert!(result.is_ok());
}

#[test]
fn parse_function_call_multiline_args() {
    let result = parse("foo(\n    1,\n    2,\n    3\n);");

    assert!(result.is_ok());
}

// =============================================================================
// Return Statement Tests
// =============================================================================

#[test]
fn parse_return_literal() {
    let result = parse("fn f() { return 42; }");

    assert!(result.is_ok());
}

#[test]
fn parse_return_expression() {
    let result = parse("fn f(x) { return x + 1; }");

    assert!(result.is_ok());
}

#[test]
fn parse_return_in_if() {
    let code = r#"
        fn abs(x) {
            if (x < 0) {
                return -x;
            } else {
                return x;
            }
        }
    "#;
    let result = parse(code);

    assert!(result.is_ok());
}

#[test]
fn parse_return_string() {
    let result = parse(r#"fn f() { return "hello"; }"#);

    assert!(result.is_ok());
}

#[test]
fn parse_return_null() {
    let result = parse("fn f() { return null; }");

    assert!(result.is_ok());
}

#[test]
fn parse_return_boolean() {
    let result = parse("fn f() { return true; }");

    assert!(result.is_ok());
}

#[test]
fn parse_return_function_call() {
    let result = parse("fn f(x) { return g(x); }");

    assert!(result.is_ok());
}

#[test]
fn eval_return_simple() {
    let code = r#"
        fn five() { return 5; }
        five();
    "#;
    let program = parse(code).unwrap();
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(5))));
}

#[test]
fn eval_return_early() {
    let code = r#"
        fn first() {
            return 1;
            return 2;
        }
        first();
    "#;
    let program = parse(code).unwrap();
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(1))));
}

// =============================================================================
// Break Statement Tests
// =============================================================================

#[test]
fn parse_break_in_while() {
    let code = r#"
        let i = 0;
        while (true) {
            if (i >= 5) { break; }
            i = i + 1;
        }
    "#;
    let result = parse(code);

    assert!(result.is_ok());
}

#[test]
fn parse_break_in_for() {
    let code = r#"
        for i in range(100) {
            if (i > 10) { break; }
        }
    "#;
    let result = parse(code);

    assert!(result.is_ok());
}

#[test]
fn eval_break_while() {
    let code = r#"
        let x = 0;
        while (true) {
            x = x + 1;
            if (x == 3) { break; }
        }
        x;
    "#;
    let program = parse(code).unwrap();
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(3))));
}

#[test]
fn eval_break_for() {
    let code = r#"
        let sum = 0;
        for i in range(100) {
            if (i >= 5) { break; }
            sum = sum + i;
        }
        sum;
    "#;
    let program = parse(code).unwrap();
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program).unwrap();
    // 0+1+2+3+4 = 10
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(10))));
}

// =============================================================================
// Continue Statement Tests
// =============================================================================

#[test]
fn parse_continue_in_while() {
    let code = r#"
        let i = 0;
        while (i < 10) {
            i = i + 1;
            if (i % 2 == 0) { continue; }
        }
    "#;
    let result = parse(code);

    assert!(result.is_ok());
}

#[test]
fn parse_continue_in_for() {
    let code = r#"
        for i in range(10) {
            if (i % 2 == 0) { continue; }
        }
    "#;
    let result = parse(code);

    assert!(result.is_ok());
}

#[test]
fn eval_continue_for() {
    let code = r#"
        let sum = 0;
        for i in range(1, 6) {
            if (i == 3) { continue; }
            sum = sum + i;
        }
        sum;
    "#;
    let program = parse(code).unwrap();
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program).unwrap();
    // 1+2+4+5 = 12
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(12))));
}

#[test]
fn eval_continue_while() {
    let code = r#"
        let sum = 0;
        let i = 0;
        while (i < 5) {
            i = i + 1;
            if (i == 3) { continue; }
            sum = sum + i;
        }
        sum;
    "#;
    let program = parse(code).unwrap();
    let context = Context::new();
    let mut axe = Axe::new(&context);
    let result = axe.run(program).unwrap();
    // 1+2+4+5 = 12
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(12))));
}

// =============================================================================
// Method Call Tests
// =============================================================================

#[test]
fn parse_method_call_no_args() {
    let result = parse("foo.bar();");

    assert!(result.is_ok());
}

#[test]
fn parse_method_call_with_args() {
    let result = parse("foo.bar(1, 2);");

    assert!(result.is_ok());
}

#[test]
fn parse_chained_method_calls() {
    let result = parse("foo.bar().baz();");

    assert!(result.is_ok());
}

#[test]
fn parse_method_call_on_string() {
    let result = parse(r#""hello".len();"#);

    assert!(result.is_ok());
}

#[test]
fn parse_method_call_after_function() {
    let result = parse("getUser().name();");

    assert!(result.is_ok());
}

#[test]
fn parse_property_access() {
    let result = parse("foo.bar;");

    assert!(result.is_ok());
}

#[test]
fn parse_chained_property_access() {
    let result = parse("foo.bar.baz;");

    assert!(result.is_ok());
}

#[test]
fn parse_property_then_method() {
    let result = parse("foo.bar.len();");

    assert!(result.is_ok());
}

#[test]
fn eval_string_len_method() {
    let code = r#""hello".len();"#;
    let result = run_code(code).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(5))));
}

#[test]
fn eval_string_concat_method() {
    let code = r#""hello".concat(" world");"#;
    let result = run_code(code).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Str(_))));
}

#[test]
fn eval_method_on_variable() {
    let code = r#"
        let s = "hello";
        s.len();
    "#;
    let result = run_code(code).unwrap();
    assert!(matches!(result, axe::Value::Literal(axe::Literal::Int(5))));
}

#[test]
fn eval_chained_method_calls() {
    let code = r#""a".concat("b").concat("c");"#;
    let result = run_code(code);
    assert!(result.is_ok());
}
