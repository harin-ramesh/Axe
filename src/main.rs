use axe::{Axe, Literal, Parser, Value};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Config, Editor, Helper};
use std::borrow::Cow;
use std::env;
use std::fs;
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

    let mut axe = Axe::new();

    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        return; // Empty file is ok
    }

    // Parse the entire file as a program
    let mut parser = Parser::new(&content);
    match parser.parse() {
        Ok(program) => match axe.run(program) {
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

/// Custom helper for rustyline with command completion and syntax highlighting
struct AxeHelper {
    commands: Vec<String>,
    keywords: Vec<String>,
}

impl AxeHelper {
    fn new() -> Self {
        Self {
            commands: vec![
                "exit".to_string(),
                "quit".to_string(),
                "help".to_string(),
                "clear".to_string(),
                "reset".to_string(),
            ],
            keywords: vec![
                "let".to_string(),
                "fn".to_string(),
                "if".to_string(),
                "else".to_string(),
                "while".to_string(),
                "for".to_string(),
                "return".to_string(),
                "true".to_string(),
                "false".to_string(),
                "null".to_string(),
                "print".to_string(),
            ],
        }
    }
}

impl Completer for AxeHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find the start of the current word
        let line_to_pos = &line[..pos];
        let word_start = line_to_pos
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &line[word_start..pos];

        if word.is_empty() {
            return Ok((pos, vec![]));
        }

        let mut candidates = Vec::new();

        // Complete commands (only at start of line)
        if word_start == 0 {
            for cmd in &self.commands {
                if cmd.starts_with(word) {
                    candidates.push(Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    });
                }
            }
        }

        // Complete keywords
        for kw in &self.keywords {
            if kw.starts_with(word) {
                candidates.push(Pair {
                    display: kw.clone(),
                    replacement: kw.clone(),
                });
            }
        }

        Ok((word_start, candidates))
    }
}

impl Hinter for AxeHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }

        let line_trimmed = line.trim();

        // Suggest completions for partial commands at start of line
        if !line_trimmed.contains(' ') && !line_trimmed.is_empty() {
            for cmd in &self.commands {
                if cmd.starts_with(line_trimmed) && cmd != line_trimmed {
                    return Some(cmd[line_trimmed.len()..].to_string());
                }
            }
        }

        None
    }
}

impl Highlighter for AxeHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Simple keyword highlighting
        let mut result = line.to_string();

        // Highlight keywords in blue
        for kw in &self.keywords {
            let pattern = format!(r"\b{}\b", kw);
            if let Ok(re) = regex::Regex::new(&pattern) {
                result = re
                    .replace_all(&result, format!("\x1b[34m{}\x1b[0m", kw))
                    .to_string();
            }
        }

        // Highlight strings in green
        if let Ok(re) = regex::Regex::new(r#""[^"]*""#) {
            result = re.replace_all(&result, "\x1b[32m$0\x1b[0m").to_string();
        }

        // Highlight numbers in yellow
        if let Ok(re) = regex::Regex::new(r"\b\d+\.?\d*\b") {
            result = re.replace_all(&result, "\x1b[33m$0\x1b[0m").to_string();
        }

        Cow::Owned(result)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Owned(format!("\x1b[1;36m{}\x1b[0m", prompt))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[2m{}\x1b[0m", hint))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }
}

impl Validator for AxeHelper {}

impl Helper for AxeHelper {}

fn get_history_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|mut path| {
        path.push("axe");
        let _ = fs::create_dir_all(&path);
        path.push("history.txt");
        path
    })
}

fn run_repl() {
    println!("\x1b[1;35mAxe Programming Language REPL\x1b[0m");
    println!(
        "\x1b[2mType 'exit' or 'quit' to exit, 'help' for help, 'clear' to clear screen\x1b[0m"
    );
    println!(
        "\x1b[2mMulti-line input supported - statements execute when braces are balanced and line ends with ';'\x1b[0m"
    );
    println!("\x1b[2mUse arrow keys for history, Tab for completion\x1b[0m");
    println!();

    let config = Config::builder()
        .auto_add_history(true)
        .max_history_size(1000)
        .expect("Invalid max_history_size")
        .build();

    let helper = AxeHelper::new();
    let mut rl: Editor<AxeHelper, _> =
        Editor::with_config(config).expect("Failed to create editor");
    rl.set_helper(Some(helper));

    // Load history
    if let Some(history_path) = get_history_path() {
        let _ = rl.load_history(&history_path);
    }

    let mut axe = Axe::new();
    let mut accumulated_input = String::new();

    loop {
        // Show different prompt based on whether we're continuing input
        let prompt = if accumulated_input.is_empty() {
            "axe> "
        } else {
            "...> "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();

                // Handle special commands (only when not in multi-line mode)
                if accumulated_input.is_empty() {
                    match trimmed {
                        "exit" | "quit" => {
                            println!("\x1b[1;33mGoodbye!\x1b[0m");
                            break;
                        }
                        "help" => {
                            print_help();
                            continue;
                        }
                        "clear" => {
                            print!("\x1b[2J\x1b[H");
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                            continue;
                        }
                        "reset" => {
                            axe = Axe::new();
                            println!("\x1b[1;32mInterpreter state reset.\x1b[0m");
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
                        Ok(program) => match axe.run(program) {
                            Ok(value) => {
                                // Only print non-null values
                                if !matches!(value, Value::Literal(Literal::Null)) {
                                    println!("\x1b[1;32m=>\x1b[0m {}", value);
                                }
                            }
                            Err(e) => println!("\x1b[1;31mError:\x1b[0m {}", e),
                        },
                        Err(e) => println!("\x1b[1;31mParse error:\x1b[0m {}", e),
                    }
                    accumulated_input.clear();
                }
                // If not complete, continue accumulating on next iteration
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: exit
                println!("\n\x1b[1;33mGoodbye!\x1b[0m");
                break;
            }
            Err(ReadlineError::Eof) => {
                // EOF: continue (ignore Ctrl-D)
                continue;
            }
            Err(err) => {
                println!("\x1b[1;31mError:\x1b[0m {:?}", err);
                break;
            }
        }
    }

    // Save history
    if let Some(history_path) = get_history_path() {
        let _ = rl.save_history(&history_path);
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
    println!("\x1b[1;35mAxe Language Help\x1b[0m");
    println!("\x1b[35m================\x1b[0m");
    println!();
    println!("\x1b[1mREPL Commands:\x1b[0m");
    println!("  \x1b[36mexit\x1b[0m, \x1b[36mquit\x1b[0m    - Exit the REPL");
    println!("  \x1b[36mhelp\x1b[0m            - Show this help");
    println!("  \x1b[36mclear\x1b[0m           - Clear the screen");
    println!("  \x1b[36mreset\x1b[0m           - Reset interpreter state");
    println!();
    println!("\x1b[1mKeyboard Shortcuts:\x1b[0m");
    println!("  \x1b[36mUp/Down\x1b[0m         - Navigate command history");
    println!("  \x1b[36mCtrl-R\x1b[0m          - Reverse search history");
    println!("  \x1b[36mTab\x1b[0m             - Auto-complete commands/keywords");
    println!("  \x1b[36mCtrl-C\x1b[0m          - Exit REPL");
    println!();
    println!("\x1b[1mSyntax:\x1b[0m C-like with semicolons and braces");
    println!();
    println!("\x1b[1mLiterals:\x1b[0m");
    println!("  \x1b[33m42\x1b[0m              - Integer");
    println!("  \x1b[33m3.14\x1b[0m            - Float");
    println!("  \x1b[32m\"hello\"\x1b[0m         - String");
    println!("  \x1b[34mtrue\x1b[0m, \x1b[34mfalse\x1b[0m     - Boolean");
    println!("  \x1b[34mnull\x1b[0m            - Null value");
    println!();
    println!("\x1b[1mArithmetic:\x1b[0m");
    println!("  1 + 2;          - Addition: \x1b[33m3\x1b[0m");
    println!("  5 - 3;          - Subtraction: \x1b[33m2\x1b[0m");
    println!("  4 * 5;          - Multiplication: \x1b[33m20\x1b[0m");
    println!("  10 / 2;         - Division: \x1b[33m5\x1b[0m");
    println!("  -x;             - Negation");
    println!();
    println!("\x1b[1mComparison:\x1b[0m");
    println!("  5 > 3;          - Greater than: \x1b[34mtrue\x1b[0m");
    println!("  2 < 4;          - Less than: \x1b[34mtrue\x1b[0m");
    println!("  5 >= 5;         - Greater or equal: \x1b[34mtrue\x1b[0m");
    println!("  3 <= 4;         - Less or equal: \x1b[34mtrue\x1b[0m");
    println!("  5 == 5;         - Equal: \x1b[34mtrue\x1b[0m");
    println!("  3 != 4;         - Not equal: \x1b[34mtrue\x1b[0m");
    println!();
    println!("\x1b[1mVariables:\x1b[0m");
    println!("  \x1b[34mlet\x1b[0m x = 10;     - Declare variable");
    println!("  x = 20;         - Update variable");
    println!("  x;              - Get variable value");
    println!();
    println!("\x1b[1mControl Flow:\x1b[0m");
    println!("  \x1b[34mif\x1b[0m (x > 0) {{ \x1b[32m\"positive\"\x1b[0m; }} \x1b[34melse\x1b[0m {{ \x1b[32m\"not positive\"\x1b[0m; }}");
    println!("  \x1b[34mwhile\x1b[0m (x > 0) {{ x = x - 1; }}");
    println!("  \x1b[34mfor\x1b[0m (\x1b[34mlet\x1b[0m i = 0; i < 10; i++) {{ ... }}");
    println!();
    println!("\x1b[1mBlocks:\x1b[0m");
    println!("  {{");
    println!("    \x1b[34mlet\x1b[0m x = \x1b[33m10\x1b[0m;");
    println!("    \x1b[34mlet\x1b[0m y = \x1b[33m20\x1b[0m;");
    println!("    x + y;");
    println!("  }}");
    println!();
    println!("\x1b[1mFunctions:\x1b[0m");
    println!("  \x1b[34mfn\x1b[0m add(x, y) {{ x + y; }}");
    println!("  add(\x1b[33m10\x1b[0m, \x1b[33m20\x1b[0m);    - Call function: \x1b[33m30\x1b[0m");
    println!();
    println!("\x1b[1mIncrement/Decrement:\x1b[0m");
    println!("  i++;            - Increment");
    println!("  i--;            - Decrement");
    println!();
    println!("\x1b[1mBuilt-in Functions:\x1b[0m");
    println!("  \x1b[34mprint\x1b[0m(\x1b[32m\"Hello\"\x1b[0m, \x1b[32m\"World\"\x1b[0m, \x1b[33m42\x1b[0m);  - Prints arguments");
    println!("  \x1b[34mlen\x1b[0m(list);      - Get list length");
    println!("  \x1b[34mget\x1b[0m(list, i);   - Get element at index");
    println!();
    println!("\x1b[1mExample:\x1b[0m");
    println!("  \x1b[34mlet\x1b[0m sum = \x1b[33m0\x1b[0m;");
    println!("  \x1b[34mfor\x1b[0m (\x1b[34mlet\x1b[0m i = \x1b[33m1\x1b[0m; i <= \x1b[33m5\x1b[0m; i++) {{");
    println!("    sum = sum + i;");
    println!("  }}");
    println!("  sum;            - Returns \x1b[33m15\x1b[0m");
    println!();
}
