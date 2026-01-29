# Axe Programming Language

A lightweight programming language interpreter written in Rust. Axe features a clean, C-like syntax with support for functions, classes, and modern control flow constructs.

## Quick Start

```bash
# Build the interpreter
cargo build --release

# Run a script
cargo run --release examples/hello.ax

# Start the REPL (interactive mode)
cargo run --release
```

## Features

- **C-like syntax** with semicolons and braces
- **Data types**: integers (i64), floats (f64), strings, booleans, null, lists
- **Variables** with block scoping and shadowing
- **Control flow**: if/else statements, while loops, for loops
- **Functions** with recursion and closures
- **Classes** with inheritance and methods
- **Built-in functions**: `print`, `type`, `range`
- **Methods** on strings and lists (`.len()`, `.concat()`, `.push()`, `.get()`)
- **Operators**: arithmetic, comparison, logical, and bitwise

## Examples

### Hello World
```javascript
print("Hello, World!");
```

### Variables and Expressions
```javascript
let x = 10;
let y = 20;
let sum = x + y;
print(sum);  // 30

// Multiple declarations
let a = 1, b = 2, c = 3;
```

### Functions
```javascript
// Define a function
fn factorial(n) {
    if (n <= 1) {
        1;
    } else {
        n * factorial(n - 1);
    }
}

print(factorial(5));  // 120

// Sum of squares using a for loop
fn sumOfSquares(limit) {
    let total = 0;
    for i in range(1, limit + 1) {
        total = total + i * i;
    }
    total;
}

print(sumOfSquares(5));  // 55
```

### Control Flow
```javascript
// If-else
if (x > 0) {
    print("positive");
} else {
    print("non-positive");
}

// While loop
let i = 0;
while (i < 5) {
    print(i);
    i = i + 1;
}

// For loop
for i in range(5) {
    print(i);  // 0, 1, 2, 3, 4
}

for i in range(1, 6) {
    print(i);  // 1, 2, 3, 4, 5
}
```

### Classes
```javascript
class Counter {
    let count = 0;
    
    fn init(self, start) {
        let count = start in self;
    }
    
    fn increment(self) {
        let count = self.count + 1 in self;
        self.count;
    }
    
    fn get(self) {
        self.count;
    }
}

let c = new Counter(0);
c.increment();
c.increment();
print(c.get());  // 2
```

### Lists
```javascript
let numbers = [1, 2, 3, 4, 5];
print(numbers.len());       // 5
print(numbers.get(0));      // 1
print(numbers.get(-1));     // 5 (negative indexing)

let more = numbers.push(6);  // [1, 2, 3, 4, 5, 6]
let combined = [1, 2].concat([3, 4]);  // [1, 2, 3, 4]
```

### Strings
```javascript
let greeting = "Hello";
print(greeting.len());                    // 5
print(greeting.concat(", World!"));       // "Hello, World!"
```

## Documentation

See the [docs](docs/index.md) folder for full documentation:

- [Getting Started](docs/getting-started.md)
- [Language Reference](docs/language-reference.md)
- [Examples](docs/examples.md)

## Example Files

The `examples/` directory contains several example programs:

| File | Description |
|------|-------------|
| `hello.ax` | Simple hello world |
| `functions.ax` | Function definitions and usage |
| `loops.ax` | While and for loop examples |
| `recursion.ax` | Recursive functions (factorial, fibonacci, etc.) |
| `lists.ax` | List creation and manipulation |
| `classes.ax` | Object-oriented programming examples |
| `builtins.ax` | Built-in operators and expressions |
| `scoping_explained.ax` | Variable scoping demonstration |

Run any example with:
```bash
cargo run --release examples/<filename>.ax
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test ast
cargo test --test parser
```

## Project Structure

```
axe/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── ast.rs            # Abstract syntax tree
│   ├── parser.rs         # Recursive descent parser
│   ├── tokeniser.rs      # Lexer/tokenizer
│   ├── transformer.rs    # AST transformer
│   └── interpreter/
│       ├── mod.rs        # Module exports
│       ├── tree_walker.rs # Interpreter implementation
│       ├── environment.rs # Variable scopes
│       ├── value.rs      # Runtime values
│       └── builtins.rs   # Native functions
├── tests/                # Integration tests
├── examples/             # Example programs
└── docs/                 # Documentation
```

## Language Overview

### Data Types

| Type | Description | Example |
|------|-------------|---------|
| Int | 64-bit integer | `42`, `-17` |
| Float | 64-bit float | `3.14`, `-0.5` |
| Str | String | `"hello"` |
| Bool | Boolean | `true`, `false` |
| Null | Null value | `null` |
| List | Dynamic array | `[1, 2, 3]` |

### Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/`, `%` |
| Comparison | `>`, `<`, `>=`, `<=`, `==`, `!=` |
| Logical | `&&`, `\|\|`, `!` |
| Bitwise | `&`, `\|`, `~` |
| Unary | `-`, `+`, `!`, `~` |

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print(value)` | Print a value to stdout |
| `type(value)` | Get the type of a value as a string |
| `range(end)` | Generate list [0, 1, ..., end-1] |
| `range(start, end)` | Generate list [start, ..., end-1] |

### Methods

**String methods:**
- `.len()` - Get string length
- `.concat(other)` - Concatenate strings

**List methods:**
- `.len()` - Get list length
- `.get(index)` - Get element at index (supports negative indexing)
- `.push(value)` - Return new list with value appended
- `.concat(other)` - Return new list with other list appended

## Roadmap

- [ ] Return statement
- [ ] Break/continue for loops
- [ ] Better error messages with line numbers
- [ ] Standard library with common utilities
- [ ] Module system with imports
- [ ] Array and map literals
- [ ] Lambda expressions
- [ ] Pattern matching
- [ ] Garbage collection
- [ ] VM bytecode compilation

## License

This project is open source. Feel free to use, modify, and distribute.
