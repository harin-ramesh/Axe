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

    // Parse and evaluate each expression in the file
    // We need to parse expressions one at a time from the file content
    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        return; // Empty file is ok
    }

    match Parser::new(&content) {
        Ok(mut parser) => {
            // Parse and execute each expression in the file
            let mut expression_count = 0;
            loop {
                // Try to parse the next expression
                match parser.parse() {
                    Ok(expr) => {
                        expression_count += 1;
                        match axe.eval(expr) {
                            Ok(_) => {} // Successfully evaluated
                            Err(e) => {
                                eprintln!(
                                    "Runtime error in expression {}: {}",
                                    expression_count, e
                                );
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        // Check if we've consumed all tokens (normal end of file)
                        if e.contains("Unexpected end of input") || e.contains("Unexpected token") {
                            break; // Normal end of file
                        }
                        eprintln!("Parse error in expression {}: {}", expression_count + 1, e);
                        process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Tokenize error: {}", e);
            process::exit(1);
        }
    }
}

fn run_repl() {
    println!("Axe Programming Language REPL");
    println!("Type 'exit' or 'quit' to exit, 'help' for help");
    println!("Multi-line input supported - expressions will execute when parentheses are balanced");
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
                    accumulated_input.push(' ');
                }
                accumulated_input.push_str(trimmed);

                // Check if parentheses are balanced
                if is_balanced(&accumulated_input) {
                    // Parse and evaluate
                    match Parser::new(&accumulated_input) {
                        Ok(mut parser) => match parser.parse() {
                            Ok(expr) => {
                                match axe.eval(expr) {
                                    // Ok(value) => println!("=> {}", value),
                                    Ok(_value) => {}
                                    Err(e) => println!("Error: {}", e),
                                }
                            }
                            Err(e) => println!("Parse error: {}", e),
                        },
                        Err(e) => println!("Tokenize error: {}", e),
                    }
                    accumulated_input.clear();
                }
                // If not balanced, continue accumulating on next iteration
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn is_balanced(input: &str) -> bool {
    let mut depth = 0;
    let mut in_string = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Toggle string mode, but handle escaped quotes
                // Simple check: if previous char is not backslash
                in_string = !in_string;
            }
            '(' if !in_string => depth += 1,
            ')' if !in_string => {
                depth -= 1;
                if depth < 0 {
                    return false; // More closing than opening
                }
            }
            _ => {}
        }
    }

    // Balanced if depth is 0 and not inside a string
    depth == 0 && !in_string
}

fn print_help() {
    println!("Axe Language Help");
    println!("================");
    println!();
    println!("REPL Features:");
    println!("  - Multi-line input: The REPL will wait for balanced parentheses");
    println!("  - Continuation prompt (...>) shown when waiting for more input");
    println!("  - Expression executes when parentheses are balanced");
    println!();
    println!("Literals:");
    println!("  42              - Integer");
    println!("  3.14            - Float");
    println!("  \"hello\"         - String");
    println!("  true, false     - Boolean");
    println!("  null            - Null value");
    println!();
    println!("Arithmetic:");
    println!("  (+ 1 2)         - Addition: 3");
    println!("  (- 5 3)         - Subtraction: 2");
    println!("  (* 4 5)         - Multiplication: 20");
    println!("  (/ 10 2)        - Division: 5");
    println!();
    println!("Comparison:");
    println!("  (> 5 3)         - Greater than: true");
    println!("  (< 2 4)         - Less than: true");
    println!("  (>= 5 5)        - Greater or equal: true");
    println!("  (<= 3 4)        - Less or equal: true");
    println!("  (== 5 5)        - Equal: true");
    println!("  (!= 3 4)        - Not equal: true");
    println!();
    println!("Variables:");
    println!("  (let x 10)      - Declare variable");
    println!("  (let x 20)   - Update variable");
    println!("  x               - Get variable value");
    println!();
    println!("Control Flow:");
    println!("  (if (> x 0) \"positive\" \"not positive\")");
    println!("  (while (> x 0) (let x (- x 1)))");
    println!();
    println!("Blocks:");
    println!("  (begin");
    println!("    (let x 10)");
    println!("    (let y 20)");
    println!("    (+ x y))");
    println!();
    println!("Functions:");
    println!("  (fn add (x y) (+ x y))");
    println!("  (add 10 20)     - Call function: 30");
    println!("  (fn factorial (n)");
    println!("    (if (<= n 1)");
    println!("      1");
    println!("      (* n (factorial (- n 1)))))");
    println!("  (factorial 5)   - Recursive call: 120");
    println!();
    println!("Built-in Functions:");
    println!("  (print \"Hello\" \"World\" 42)  - Prints arguments");
    println!();
    println!("Examples:");
    println!("  (let sum 0)");
    println!("  (let i 1)");
    println!("  (while (<= i 5)");
    println!("    (let sum (+ sum i))");
    println!("    (let i (+ i 1)))");
    println!("  sum             - Returns 15");
    println!();
}
