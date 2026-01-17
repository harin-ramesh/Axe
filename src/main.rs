use axe::{Axe, Parser};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if a file argument was provided
    if args.len() > 1 {
        run_file(&args[1]);
        return;
    }

    // Otherwise, start REPL
    run_repl();
}

fn run_file(filename: &str) {
    // Read the file
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let axe = Axe::new();

    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        return; // Empty file is ok
    }

    // Parse the entire file as a program
    let mut parser = Parser::new(&content);
    match parser.parse() {
        Ok(expr) => match axe.eval(expr) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Runtime error: {}", e);
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    }
}

fn run_repl() {
    println!("Axe Programming Language REPL");
    println!("Type 'exit' or 'quit' to exit, 'help' for help");
    println!(
        "Multi-line input supported - statements execute when braces are balanced and line ends with ';'"
    );
    println!();

    let axe = Axe::new();
    let mut line = String::new();
    let mut accumulated_input = String::new();

    loop {
        // Show different prompt based on whether we're continuing input
        if accumulated_input.is_empty() {
            print!("axe> ");
        } else {
            print!("...> ");
        }
        io::stdout().flush().unwrap();

        line.clear();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                // EOF
                println!();
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();

                // Handle special commands (only when not in multi-line mode)
                if accumulated_input.is_empty() {
                    match trimmed {
                        "exit" | "quit" => {
                            println!("Goodbye!");
                            break;
                        }
                        "help" => {
                            print_help();
                            continue;
                        }
                        "" => continue,
                        _ => {}
                    }
                }

                // Accumulate input
                if !accumulated_input.is_empty() {
                    accumulated_input.push('\n');
                }
                accumulated_input.push_str(trimmed);

                // Check if input is complete (braces balanced and ends with semicolon or closing brace)
                if is_complete(&accumulated_input) {
                    // Parse and evaluate
                    let mut parser = Parser::new(&accumulated_input);
                    match parser.parse() {
                        Ok(expr) => match axe.eval(expr) {
                            Ok(value) => {
                                // Only print non-null values
                                if !matches!(value, axe::Value::Null) {
                                    println!("=> {}", value);
                                }
                            }
                            Err(e) => println!("Error: {}", e),
                        },
                        Err(e) => println!("Parse error: {}", e),
                    }
                    accumulated_input.clear();
                }
                // If not complete, continue accumulating on next iteration
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }
}

/// Check if the input is complete and ready to execute.
/// Input is complete when:
/// - Braces {} and parentheses () are balanced
/// - Not inside a string
/// - Ends with ';' or '}'
fn is_complete(input: &str) -> bool {
    let mut brace_depth = 0;
    let mut paren_depth = 0;
    let mut in_string = false;
    let mut prev_char = ' ';

    for ch in input.chars() {
        if in_string {
            if ch == '"' && prev_char != '\\' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    if brace_depth < 0 {
                        return false;
                    }
                }
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth < 0 {
                        return false;
                    }
                }
                _ => {}
            }
        }
        prev_char = ch;
    }

    // Must be balanced and not inside a string
    if brace_depth != 0 || paren_depth != 0 || in_string {
        return false;
    }

    // Check if ends with semicolon or closing brace (ignoring trailing whitespace)
    let trimmed = input.trim();
    trimmed.ends_with(';') || trimmed.ends_with('}')
}

fn print_help() {
    println!("Axe Language Help");
    println!("================");
    println!();
    println!("Syntax: C-like with semicolons and braces");
    println!();
    println!("Literals:");
    println!("  42              - Integer");
    println!("  3.14            - Float");
    println!("  \"hello\"         - String");
    println!("  true, false     - Boolean");
    println!("  null            - Null value");
    println!();
    println!("Arithmetic:");
    println!("  1 + 2;          - Addition: 3");
    println!("  5 - 3;          - Subtraction: 2");
    println!("  4 * 5;          - Multiplication: 20");
    println!("  10 / 2;         - Division: 5");
    println!("  -x;             - Negation");
    println!();
    println!("Comparison:");
    println!("  5 > 3;          - Greater than: true");
    println!("  2 < 4;          - Less than: true");
    println!("  5 >= 5;         - Greater or equal: true");
    println!("  3 <= 4;         - Less or equal: true");
    println!("  5 == 5;         - Equal: true");
    println!("  3 != 4;         - Not equal: true");
    println!();
    println!("Variables:");
    println!("  let x = 10;     - Declare variable");
    println!("  x = 20;         - Update variable");
    println!("  x;              - Get variable value");
    println!();
    println!("Control Flow:");
    println!("  if (x > 0) {{ \"positive\"; }} else {{ \"not positive\"; }}");
    println!("  while (x > 0) {{ x = x - 1; }}");
    println!("  for (let i = 0; i < 10; i++) {{ ... }}");
    println!();
    println!("Blocks:");
    println!("  {{");
    println!("    let x = 10;");
    println!("    let y = 20;");
    println!("    x + y;");
    println!("  }}");
    println!();
    println!("Functions:");
    println!("  fn add(x, y) {{ x + y; }}");
    println!("  add(10, 20);    - Call function: 30");
    println!();
    println!("Increment/Decrement:");
    println!("  i++;            - Increment");
    println!("  i--;            - Decrement");
    println!();
    println!("Built-in Functions:");
    println!("  print(\"Hello\", \"World\", 42);  - Prints arguments");
    println!("  len(list);      - Get list length");
    println!("  get(list, i);   - Get element at index");
    println!();
    println!("Example:");
    println!("  let sum = 0;");
    println!("  for (let i = 1; i <= 5; i++) {{");
    println!("    sum = sum + i;");
    println!("  }}");
    println!("  sum;            - Returns 15");
    println!();
}
