# Axe Programming Language

A lightweight S-expression based programming language interpreter written in Rust, featuring variables, control flow, and proper scoping.

## Features

- **S-Expression Syntax**: Clean Lisp-like syntax
- **Arithmetic Operations**: `+`, `-`, `*`, `/` for integers and floats
- **Comparison Operations**: `>`, `<`, `>=`, `<=`, `==`, `!=`
- **Variables**: `set` for declaration, `assign` for updates
- **Control Flow**: `if` expressions and `while` loops
- **Block Scoping**: Lexical scoping with blocks
- **Boolean Type**: `true` and `false` with proper truthiness rules
- **Type Safety**: Strong type checking at runtime
- **Interactive REPL**: Command-line interface for experimentation

## Installation

```bash
cargo build --release
```

## Usage

### Interactive REPL

Start the REPL:

```bash
cargo run --release
```

Example session:

```
Axe Programming Language REPL
Type 'exit' or 'quit' to exit, 'help' for help

axe> (+ 10 20)
=> Int(30)

axe> (set x 5)
=> Int(5)

axe> (* x 2)
=> Int(10)

axe> (if (> x 3) "big" "small")
=> Str("big")

axe> quit
Goodbye!
```

### As a Library

```rust
use axe::{Axe, Parser};

fn main() {
    let axe = Axe::new();
    
    // Parse and evaluate
    let mut parser = Parser::new("(+ 10 20)").unwrap();
    let expr = parser.parse().unwrap();
    let result = axe.eval(expr).unwrap();
    println!("{:?}", result); // Int(30)
}
```

## Language Syntax

### Literals

```lisp
42                  ; Integer
3.14                ; Float
"hello"             ; String
true                ; Boolean true
false               ; Boolean false
null                ; Null value
```

### Arithmetic

```lisp
(+ 1 2)             ; 3
(- 5 3)             ; 2
(* 4 5)             ; 20
(/ 10 2)            ; 5
(+ (* 2 3) 4)       ; 10 (nested expressions)
```

### Comparisons

```lisp
(> 5 3)             ; true
(< 2 4)             ; true
(>= 5 5)            ; true
(<= 3 4)            ; true
(== 5 5)            ; true
(!= 3 4)            ; true
```

### Variables

```lisp
(set x 10)          ; Declare variable x = 10
x                   ; Get value: 10
(assign x 20)       ; Update x = 20
(set y (+ x 5))     ; y = 25
```

### Control Flow

#### If Expressions

```lisp
(if (> x 0) "positive" "not positive")

(if (== x 0)
    "zero"
    "not zero")
```

#### While Loops

```lisp
(set counter 5)
(while (> counter 0)
    (assign counter (- counter 1)))

; Multiple statements in while
(while (< i 10)
    (assign sum (+ sum i))
    (assign i (+ i 1)))
```

### Blocks

Blocks create new scopes:

```lisp
(block
    (set x 10)
    (set y 20)
    (+ x y))        ; Returns 30
```

### Complete Example: Sum from 1 to 5

```lisp
(block
    (set sum 0)
    (set i 1)
    (while (<= i 5)
        (assign sum (+ sum i))
        (assign i (+ i 1)))
    sum)            ; Returns 15
```

## Expression Types

### Literals
- `Null` - Null value
- `Bool(bool)` - Boolean
- `Int(i64)` - Integer  
- `Float(f64)` - Float
- `Str(String)` - String

### Operations
- `Binary(op, left, right)` - Binary operations
- `Set(name, expr)` - Variable declaration
- `Assign(name, expr)` - Variable assignment
- `Var(name)` - Variable reference
- `Block(exprs)` - Block with new scope
- `If(condition, then, else)` - Conditional
- `While(condition, body)` - Loop

## Truthiness Rules

Falsy values:
- `null`
- `false`
- `0` (integer zero)
- `0.0` (float zero)

Everything else is truthy (including empty strings).

## Scoping Rules

Axe implements lexical scoping:

### Variable Access
```lisp
(set x 10)
(block
    x)              ; Can access parent scope: 10
```

### Variable Shadowing
```lisp
(set x 1)
(block
    (set x 100)     ; Shadows parent x
    x)              ; 100
x                   ; Still 1 (parent unchanged)
```

### Scope Isolation
```lisp
(block
    (set local 42))
local               ; Error: undefined variable
```

## REPL Commands

- `help` - Show help message
- `exit` or `quit` - Exit the REPL
- Any Axe expression - Evaluate and print result

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test parser_tests
cargo test --test while_tests
cargo test --test if_tests
cargo test --test comparison_tests
cargo test --test block_tests
```

### Test Coverage

113 tests covering:
- **Parser**: 24 tests - S-expression parsing
- **While Loops**: 10 tests - Loop execution
- **If Expressions**: 18 tests - Conditionals
- **Comparisons**: 29 tests - All comparison operators
- **Blocks**: 11 tests - Scoping and nesting
- **Variables**: 9 tests - Variable operations
- **Evaluation**: 4 tests - Basic evaluation
- **Assignments**: 8 tests - Variable updates

## Error Handling

Descriptive error messages:
- `"undefined variable"` - Variable not found
- `"invalid variable name"` - Invalid identifier
- `"division by zero"` - Division by zero
- `"type error"` - Type mismatch
- `"Unterminated string"` - Parse error
- `"Expected operator"` - Syntax error

## Architecture

```
axe/
├── src/
│   ├── lib.rs      - Core interpreter and evaluation
│   ├── parser.rs   - S-expression parser
│   └── main.rs     - REPL implementation
├── tests/
│   ├── parser_tests.rs
│   ├── while_tests.rs
│   ├── if_tests.rs
│   ├── comparison_tests.rs
│   ├── block_tests.rs
│   ├── variable_tests.rs
│   └── ... (8 test files total)
└── Cargo.toml
```

## Implementation Details

### Environment Model

Uses Rc<RefCell<Environment>> for shared ownership with interior mutability:

```rust
type EnvRef = Rc<RefCell<Environment>>;

pub struct Environment {
    records: HashMap<String, Value>,
    parent: Option<EnvRef>,
}
```

### Variable Validation

Variable names must:
- Start with letter (a-z, A-Z) or underscore (_)
- Contain only letters, digits, or underscores
- Match regex: `^[a-zA-Z_][a-zA-Z0-9_]*$`

