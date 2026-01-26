# Axe Programming Language

A lightweight programming language interpreter written in Rust.

## Quick Start

```bash
# Build
cargo build --release

# Run an example
cargo run --release examples/hello.ax
```

## Documentation

See the [docs](docs/index.md) folder for full documentation:

- [Getting Started](docs/getting-started.md)
- [Language Reference](docs/language-reference.md)
- [Examples](docs/examples.md)

## Example

```javascript
// Recursive factorial function
fn factorial(n) {
    if (n <= 1) {
        1;
    } else {
        n * factorial(n - 1);
    }
}

// Calculate and print factorial of 5
let result = factorial(5);
print(result);  // 120

// Higher-order computation: sum of squares
fn sumOfSquares(limit) {
    let total = 0;
    for i in range(1, limit + 1) {
        total = total + i * i;
    }
    total;
}

print(sumOfSquares(5));  // 55 (1 + 4 + 9 + 16 + 25)
```

## Features

- C-like syntax
- Variables and block scoping
- Arithmetic, comparison, logical, and bitwise operators
- Control flow with if/else, while, and for loops
- Functions with recursion support
- Data types: int, float, string, bool, null
