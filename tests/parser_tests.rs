use axe::{Expr, Operation, Parser};

// =============================================================================
// Numeric Literal Tests - These should pass with current implementation
// =============================================================================

#[test]
fn parse_integer_literal() {
    let mut parser = Parser::new("42;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_negative_integer() {
    let mut parser = Parser::new("-42;");
    let expr = parser.parse().unwrap();
    // Unary minus is represented as (0 - 42)
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(0)),
            Box::new(Expr::Int(42))
        )])
    );
}

#[test]
fn parse_positive_integer() {
    let mut parser = Parser::new("+42;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_zero() {
    let mut parser = Parser::new("0;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(0)]));
}

#[test]
fn parse_large_integer() {
    let mut parser = Parser::new("9999999999;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(9999999999)]));
}

#[test]
fn parse_float_literal() {
    let mut parser = Parser::new("3.14;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Float(3.14)]));
}

#[test]
fn parse_negative_float() {
    let mut parser = Parser::new("-3.14;");
    let expr = parser.parse().unwrap();
    // Unary minus is represented as (0 - 3.14)
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(0)),
            Box::new(Expr::Float(3.14))
        )])
    );
}

#[test]
fn parse_float_with_trailing_zeros() {
    let mut parser = Parser::new("1.500;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Float(1.5)]));
}

#[test]
fn parse_float_whole_number() {
    let mut parser = Parser::new("5.0;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Float(5.0)]));
}

// =============================================================================
// String Literal Tests - These should pass with current implementation
// =============================================================================

#[test]
fn parse_simple_string() {
    let mut parser = Parser::new("\"hello\";");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Str("hello".to_string())]));
}

#[test]
fn parse_empty_string() {
    let mut parser = Parser::new("\"\";");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Str("".to_string())]));
}

#[test]
fn parse_string_with_spaces() {
    let mut parser = Parser::new("\"hello world\";");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Str("hello world".to_string())])
    );
}

#[test]
fn parse_string_with_numbers() {
    let mut parser = Parser::new("\"test123\";");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Str("test123".to_string())]));
}

#[test]
fn parse_string_with_special_chars() {
    let mut parser = Parser::new("\"!@#$%^&*()\";");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Str("!@#$%^&*()".to_string())]));
}

#[test]
fn parse_string_with_escaped_quote() {
    let mut parser = Parser::new(r#""hello\"world";"#);
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Str(r#"hello\"world"#.to_string())])
    );
}

#[test]
fn parse_string_with_escaped_newline() {
    let mut parser = Parser::new(r#""hello\nworld";"#);
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Str(r#"hello\nworld"#.to_string())])
    );
}

// =============================================================================
// Expression Statement Tests (with semicolons)
// =============================================================================

#[test]
fn parse_single_expression_statement() {
    let mut parser = Parser::new("100;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(100)]));
}

#[test]
fn parse_string_expression_statement() {
    let mut parser = Parser::new("\"test\";");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Str("test".to_string())]));
}

#[test]
fn parse_float_expression_statement() {
    let mut parser = Parser::new("2.718;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Float(2.718)]));
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
    eprintln!("{:?}", result);
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
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Str(long_string)]));
}

#[test]
fn parse_unicode_string() {
    let mut parser = Parser::new("\"hello unicode\";");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Str("hello unicode".to_string())])
    );
}

// =============================================================================
// Block Statement Tests
// =============================================================================

#[test]
fn parse_empty_block() {
    let mut parser = Parser::new("{}");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Block(vec![])]));
}

#[test]
fn parse_block_with_single_statement() {
    let mut parser = Parser::new("{ 42; }");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Block(vec![Expr::Int(42)])]));
}

#[test]
fn parse_block_with_string() {
    let mut parser = Parser::new("{ \"hello\"; }");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![Expr::Str("hello".to_string())])])
    );
}

#[test]
fn parse_block_with_formatting() {
    let mut parser = Parser::new("{\n    42;\n}");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Block(vec![Expr::Int(42)])]));
}

// =============================================================================
// Whitespace Tests
// =============================================================================

#[test]
fn parse_with_leading_whitespace() {
    let mut parser = Parser::new("   42;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_with_trailing_whitespace() {
    let mut parser = Parser::new("42;   ");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_with_newlines() {
    let mut parser = Parser::new("\n42;\n");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_with_tabs() {
    let mut parser = Parser::new("\t42;\t");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

// =============================================================================
// Comment Tests
// =============================================================================

#[test]
fn parse_with_line_comment() {
    let mut parser = Parser::new("// this is a comment\n42;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_with_inline_comment() {
    let mut parser = Parser::new("42; // inline comment");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_with_block_comment() {
    let mut parser = Parser::new("/* block comment */ 42;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

#[test]
fn parse_with_multiline_block_comment() {
    let mut parser = Parser::new("/*\n  multiline\n  comment\n*/ 42;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42)]));
}

// =============================================================================
// Multiple Statements Tests
// =============================================================================

#[test]
fn parse_two_statements() {
    let mut parser = Parser::new("42; 100;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42), Expr::Int(100)]));
}

#[test]
fn parse_three_statements() {
    let mut parser = Parser::new("1; 2; 3;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)])
    );
}

#[test]
#[ignore = "multiple statements not yet implemented"]
fn parse_mixed_type_statements() {
    let mut parser = Parser::new("42; \"hello\"; 3.14;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![
            Expr::Int(42),
            Expr::Str("hello".to_string()),
            Expr::Float(3.14)
        ])
    );
}

#[test]
fn parse_statements_on_multiple_lines() {
    let mut parser = Parser::new("42;\n100;\n200;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Int(42), Expr::Int(100), Expr::Int(200)])
    );
}

#[test]
fn parse_statements_with_blank_lines() {
    let mut parser = Parser::new("42;\n\n100;");
    let expr = parser.parse().unwrap();
    assert_eq!(expr, Expr::Block(vec![Expr::Int(42), Expr::Int(100)]));
}

#[test]
fn parse_nested_empty_blocks() {
    let mut parser = Parser::new("{ {} }");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![Expr::Block(vec![])])])
    );
}

#[test]
fn parse_nested_block_with_statement() {
    let mut parser = Parser::new("{ { 42; } }");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![Expr::Block(vec![Expr::Int(42)])])])
    );
}

#[test]
fn parse_deeply_nested_blocks() {
    let mut parser = Parser::new("{ { { 42; } } }");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![Expr::Block(vec![Expr::Block(
            vec![Expr::Int(42)]
        )])])])
    );
}

#[test]
fn parse_sibling_blocks() {
    let mut parser = Parser::new("{ { 1; } { 2; } }");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![
            Expr::Block(vec![Expr::Int(1)]),
            Expr::Block(vec![Expr::Int(2)])
        ])])
    );
}

#[test]
fn parse_block_with_multiple_statements() {
    let mut parser = Parser::new("{ 1; 2; 3; }");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![
            Expr::Int(1),
            Expr::Int(2),
            Expr::Int(3)
        ])])
    );
}

#[test]
fn parse_nested_block_with_multiple_statements() {
    let mut parser = Parser::new("{ 1; { 2; 3; } 4; }");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![
            Expr::Int(1),
            Expr::Block(vec![Expr::Int(2), Expr::Int(3)]),
            Expr::Int(4)
        ])])
    );
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
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Block(vec![
            Expr::Int(1),
            Expr::Block(vec![Expr::Int(2), Expr::Block(vec![Expr::Int(3)])]),
            Expr::Int(4)
        ])])
    );
}

// =============================================================================
// Addition Expression Tests
// =============================================================================

#[test]
fn parse_simple_addition() {
    let mut parser = Parser::new("1 + 2;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Int(1)),
            Box::new(Expr::Int(2))
        )])
    );
}

#[test]
fn parse_chained_addition() {
    let mut parser = Parser::new("1 + 2 + 3;");
    let expr = parser.parse().unwrap();
    // Left-associative: (1 + 2) + 3
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(1)),
                Box::new(Expr::Int(2))
            )),
            Box::new(Expr::Int(3))
        )])
    );
}

#[test]
fn parse_parenthesized_addition() {
    let mut parser = Parser::new("(1 + 2) + 3;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(1)),
                Box::new(Expr::Int(2))
            )),
            Box::new(Expr::Int(3))
        )])
    );
}

#[test]
fn parse_right_grouped_addition() {
    let mut parser = Parser::new("1 + (2 + 3);");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Int(1)),
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(2)),
                Box::new(Expr::Int(3))
            ))
        )])
    );
}

#[test]
fn parse_addition_no_spaces() {
    let mut parser = Parser::new("1+2;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Int(1)),
            Box::new(Expr::Int(2))
        )])
    );
}

#[test]
fn parse_addition_with_floats() {
    let mut parser = Parser::new("1.5 + 2.5;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Float(1.5)),
            Box::new(Expr::Float(2.5))
        )])
    );
}

#[test]
fn parse_nested_parentheses() {
    let mut parser = Parser::new("((1 + 2));");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Int(1)),
            Box::new(Expr::Int(2))
        )])
    );
}

// =============================================================================
// Addition Evaluation Tests (end-to-end)
// =============================================================================

#[test]
fn eval_simple_addition() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("1 + 2;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn eval_chained_addition() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("1 + 2 + 3;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(6));
}

#[test]
fn eval_parenthesized_addition() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("(10 + 20) + 30;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(60));
}

#[test]
fn eval_float_addition() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("1.5 + 2.5;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Float(4.0));
}

// =============================================================================
// Subtraction Expression Tests
// =============================================================================

#[test]
fn parse_simple_subtraction() {
    let mut parser = Parser::new("5 - 3;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(5)),
            Box::new(Expr::Int(3))
        )])
    );
}

#[test]
fn parse_chained_subtraction() {
    let mut parser = Parser::new("10 - 3 - 2;");
    let expr = parser.parse().unwrap();
    // Left-associative: (10 - 3) - 2
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Int(10)),
                Box::new(Expr::Int(3))
            )),
            Box::new(Expr::Int(2))
        )])
    );
}

#[test]
fn parse_subtraction_with_parentheses() {
    let mut parser = Parser::new("10 - (3 - 2);");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(10)),
            Box::new(Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Int(3)),
                Box::new(Expr::Int(2))
            ))
        )])
    );
}

#[test]
fn parse_mixed_addition_subtraction() {
    let mut parser = Parser::new("1 + 2 - 3;");
    let expr = parser.parse().unwrap();
    // Left-associative: (1 + 2) - 3
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(1)),
                Box::new(Expr::Int(2))
            )),
            Box::new(Expr::Int(3))
        )])
    );
}

#[test]
fn parse_subtraction_no_spaces() {
    let mut parser = Parser::new("5-3;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(5)),
            Box::new(Expr::Int(3))
        )])
    );
}

// =============================================================================
// Multiplication Expression Tests
// =============================================================================

#[test]
fn parse_simple_multiplication() {
    let mut parser = Parser::new("3 * 4;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Int(3)),
            Box::new(Expr::Int(4))
        )])
    );
}

#[test]
fn parse_chained_multiplication() {
    let mut parser = Parser::new("2 * 3 * 4;");
    let expr = parser.parse().unwrap();
    // Left-associative: (2 * 3) * 4
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Int(2)),
                Box::new(Expr::Int(3))
            )),
            Box::new(Expr::Int(4))
        )])
    );
}

#[test]
fn parse_multiplication_with_parentheses() {
    let mut parser = Parser::new("2 * (3 * 4);");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Int(2)),
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Int(3)),
                Box::new(Expr::Int(4))
            ))
        )])
    );
}

#[test]
fn parse_multiplication_no_spaces() {
    let mut parser = Parser::new("3*4;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Int(3)),
            Box::new(Expr::Int(4))
        )])
    );
}

#[test]
fn parse_multiplication_with_floats() {
    let mut parser = Parser::new("2.5 * 4.0;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Float(2.5)),
            Box::new(Expr::Float(4.0))
        )])
    );
}

// =============================================================================
// Division Expression Tests
// =============================================================================

#[test]
fn parse_simple_division() {
    let mut parser = Parser::new("10 / 2;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Div,
            Box::new(Expr::Int(10)),
            Box::new(Expr::Int(2))
        )])
    );
}

#[test]
fn parse_chained_division() {
    let mut parser = Parser::new("100 / 10 / 2;");
    let expr = parser.parse().unwrap();
    // Left-associative: (100 / 10) / 2
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Div,
            Box::new(Expr::Binary(
                Operation::Div,
                Box::new(Expr::Int(100)),
                Box::new(Expr::Int(10))
            )),
            Box::new(Expr::Int(2))
        )])
    );
}

#[test]
fn parse_division_with_parentheses() {
    let mut parser = Parser::new("100 / (10 / 2);");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Div,
            Box::new(Expr::Int(100)),
            Box::new(Expr::Binary(
                Operation::Div,
                Box::new(Expr::Int(10)),
                Box::new(Expr::Int(2))
            ))
        )])
    );
}

#[test]
fn parse_mixed_multiplication_division() {
    let mut parser = Parser::new("2 * 6 / 3;");
    let expr = parser.parse().unwrap();
    // Left-associative: (2 * 6) / 3
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Div,
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Int(2)),
                Box::new(Expr::Int(6))
            )),
            Box::new(Expr::Int(3))
        )])
    );
}

#[test]
fn parse_division_no_spaces() {
    let mut parser = Parser::new("10/2;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Div,
            Box::new(Expr::Int(10)),
            Box::new(Expr::Int(2))
        )])
    );
}

// =============================================================================
// Operator Precedence Tests (Multiplicative binds tighter than Additive)
// =============================================================================

#[test]
fn parse_precedence_mul_over_add() {
    let mut parser = Parser::new("1 + 2 * 3;");
    let expr = parser.parse().unwrap();
    // Should parse as: 1 + (2 * 3)
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Int(1)),
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Int(2)),
                Box::new(Expr::Int(3))
            ))
        )])
    );
}

#[test]
fn parse_precedence_div_over_sub() {
    let mut parser = Parser::new("10 - 6 / 2;");
    let expr = parser.parse().unwrap();
    // Should parse as: 10 - (6 / 2)
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(10)),
            Box::new(Expr::Binary(
                Operation::Div,
                Box::new(Expr::Int(6)),
                Box::new(Expr::Int(2))
            ))
        )])
    );
}

#[test]
fn parse_precedence_mul_before_sub() {
    let mut parser = Parser::new("2 * 3 - 1;");
    let expr = parser.parse().unwrap();
    // Should parse as: (2 * 3) - 1
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Int(2)),
                Box::new(Expr::Int(3))
            )),
            Box::new(Expr::Int(1))
        )])
    );
}

#[test]
fn parse_precedence_override_with_parentheses() {
    let mut parser = Parser::new("(1 + 2) * 3;");
    let expr = parser.parse().unwrap();
    // Parentheses override: (1 + 2) * 3
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(1)),
                Box::new(Expr::Int(2))
            )),
            Box::new(Expr::Int(3))
        )])
    );
}

#[test]
fn parse_complex_precedence() {
    let mut parser = Parser::new("1 + 2 * 3 - 4 / 2;");
    let expr = parser.parse().unwrap();
    // Should parse as: (1 + (2 * 3)) - (4 / 2)
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(1)),
                Box::new(Expr::Binary(
                    Operation::Mul,
                    Box::new(Expr::Int(2)),
                    Box::new(Expr::Int(3))
                ))
            )),
            Box::new(Expr::Binary(
                Operation::Div,
                Box::new(Expr::Int(4)),
                Box::new(Expr::Int(2))
            ))
        )])
    );
}

#[test]
fn parse_all_operators_chained() {
    let mut parser = Parser::new("10 + 5 - 3 * 2 / 1;");
    let expr = parser.parse().unwrap();
    // Should parse as: (10 + 5) - ((3 * 2) / 1)
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(10)),
                Box::new(Expr::Int(5))
            )),
            Box::new(Expr::Binary(
                Operation::Div,
                Box::new(Expr::Binary(
                    Operation::Mul,
                    Box::new(Expr::Int(3)),
                    Box::new(Expr::Int(2))
                )),
                Box::new(Expr::Int(1))
            ))
        )])
    );
}

// =============================================================================
// Unary Operator Tests (parse_primary unary +/-)
// =============================================================================

#[test]
fn parse_unary_minus_in_expression() {
    let mut parser = Parser::new("5 + -3;");
    let expr = parser.parse().unwrap();
    // -3 is represented as (0 - 3)
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Int(5)),
            Box::new(Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Int(0)),
                Box::new(Expr::Int(3))
            ))
        )])
    );
}

#[test]
fn parse_unary_plus_in_expression() {
    let mut parser = Parser::new("5 + +3;");
    let expr = parser.parse().unwrap();
    // +3 is just 3
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Add,
            Box::new(Expr::Int(5)),
            Box::new(Expr::Int(3))
        )])
    );
}

#[test]
fn parse_double_unary_minus_with_parens() {
    let mut parser = Parser::new("-(-5);");
    let expr = parser.parse().unwrap();
    // -(-5) is (0 - (0 - 5))
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(0)),
            Box::new(Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Int(0)),
                Box::new(Expr::Int(5))
            ))
        )])
    );
}

#[test]
fn parse_unary_minus_with_parentheses() {
    let mut parser = Parser::new("-(1 + 2);");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Int(0)),
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(1)),
                Box::new(Expr::Int(2))
            ))
        )])
    );
}

#[test]
fn parse_unary_in_multiplicative() {
    let mut parser = Parser::new("2 * -3;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Int(2)),
            Box::new(Expr::Binary(
                Operation::Sub,
                Box::new(Expr::Int(0)),
                Box::new(Expr::Int(3))
            ))
        )])
    );
}

// =============================================================================
// Evaluation Tests for Arithmetic Operations
// =============================================================================

#[test]
fn eval_simple_subtraction() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("10 - 3;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(7));
}

#[test]
fn eval_chained_subtraction() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("10 - 3 - 2;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(5)); // (10 - 3) - 2 = 5
}

#[test]
fn eval_simple_multiplication() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("3 * 4;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(12));
}

#[test]
fn eval_simple_division() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("10 / 2;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn eval_precedence_mul_over_add() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("1 + 2 * 3;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(7)); // 1 + (2 * 3) = 7
}

#[test]
fn eval_precedence_with_parentheses() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("(1 + 2) * 3;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(9)); // (1 + 2) * 3 = 9
}

#[test]
fn eval_complex_expression() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("2 + 3 * 4 - 10 / 2;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(9)); // 2 + 12 - 5 = 9
}

#[test]
fn eval_unary_minus_in_expression() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("5 + -3;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(2));
}

#[test]
fn eval_unary_minus_with_multiplication() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("2 * -3;");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(-6));
}

#[test]
fn eval_nested_parentheses_complex() {
    use axe::{Axe, Value};

    let mut parser = Parser::new("((2 + 3) * (4 - 1));");
    let expr = parser.parse().unwrap();

    let axe = Axe::new();
    let result = axe.eval(expr).unwrap();
    assert_eq!(result, Value::Int(15)); // (5) * (3) = 15
}

// =============================================================================
// Assignment Tests
// ============================================================================

#[test]
fn parse_simple_assignment() {
    let mut parser = Parser::new("x = 42;");
    let expr = parser.parse().unwrap();
    // The `=` operator is reassignment, which uses Expr::Assign
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Assign("x".to_string(), Box::new(Expr::Int(42)))])
    );
}

#[test]
fn parse_let_declaration() {
    let mut parser = Parser::new("let x = 42;");
    let expr = parser.parse().unwrap();
    // The `let` keyword creates a new variable, which uses Expr::Let containing Expr::Set
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Let(vec![Expr::Set(
            "x".to_string(),
            Box::new(Expr::Int(42))
        )])])
    );
}

#[test]
fn parse_let_with_expression() {
    let mut parser = Parser::new("let x = 1 + 2;");
    let expr = parser.parse().unwrap();
    assert_eq!(
        expr,
        Expr::Block(vec![Expr::Let(vec![Expr::Set(
            "x".to_string(),
            Box::new(Expr::Binary(
                Operation::Add,
                Box::new(Expr::Int(1)),
                Box::new(Expr::Int(2))
            ))
        )])])
    );
}
