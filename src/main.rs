use eva::{Eva, Parser};
use std::io::{self, Write};

fn main() {
    println!("Eva Programming Language REPL");
    println!("Type 'exit' or 'quit' to exit, 'help' for help");
    println!();

    let eva = Eva::new();
    let mut input = String::new();

    loop {
        print!("eva> ");
        io::stdout().flush().unwrap();

        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();

                // Handle special commands
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

                // Parse and evaluate
                match Parser::new(trimmed) {
                    Ok(mut parser) => match parser.parse() {
                        Ok(expr) => match eva.eval(expr) {
                            Ok(value) => println!("=> {:?}", value),
                            Err(e) => println!("Error: {}", e),
                        },
                        Err(e) => println!("Parse error: {}", e),
                    },
                    Err(e) => println!("Tokenize error: {}", e),
                }
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn print_help() {
    println!("Eva Language Help");
    println!("================");
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
    println!("  (set x 10)      - Declare variable");
    println!("  (assign x 20)   - Update variable");
    println!("  x               - Get variable value");
    println!();
    println!("Control Flow:");
    println!("  (if (> x 0) \"positive\" \"not positive\")");
    println!("  (while (> x 0) (assign x (- x 1)))");
    println!();
    println!("Blocks:");
    println!("  (block");
    println!("    (set x 10)");
    println!("    (set y 20)");
    println!("    (+ x y))");
    println!();
    println!("Examples:");
    println!("  (set sum 0)");
    println!("  (set i 1)");
    println!("  (while (<= i 5)");
    println!("    (assign sum (+ sum i))");
    println!("    (assign i (+ i 1)))");
    println!("  sum             - Returns 15");
    println!();
}
