# Getting Started

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

## Installation

```bash
git clone <repository-url>
cd axe
cargo build --release
```

The binary will be at `./target/release/axe`.

## Running Programs

```bash
# Using cargo
cargo run --release <filename>.ax

# Using compiled binary
./target/release/axe <filename>.ax
```

## Your First Program

Create `hello.ax`:

```javascript
// My first Axe program
let message = "Hello, World!";
let x = 10;
let y = 20;
let sum = x + y;
```

Run it:

```bash
cargo run --release hello.ax
```

## Using Loops

Create `loops.ax`:

```javascript
// While loop - sum 1 to 10
let sum = 0;
let i = 1;
while (i <= 10) {
    sum = sum + i;
    i = i + 1;
}

// For loop - same result
let sum2 = 0;
for n in range(1, 11) {
    sum2 = sum2 + n;
}
```

## Defining Functions

Create `functions.ax`:

```javascript
// Define a function
fn square(x) {
    x * x;
}

// Function with multiple parameters
fn add(a, b) {
    a + b;
}

// Recursive function
fn factorial(n) {
    if (n <= 1) {
        1;
    } else {
        n * factorial(n - 1);
    }
}

// Call the functions
let result = square(5);       // 25
let sum = add(10, 20);        // 30
let fact5 = factorial(5);     // 120
```

## Example Files

The `examples/` directory contains sample programs:

| File | Description |
|------|-------------|
| `hello.ax` | Hello world |
| `fibonacci.ax` | Arithmetic sequences |
| `simple_counter.ax` | Variable assignment |
| `builtins.ax` | Operators demo |
| `scoping_explained.ax` | Block scoping guide |

```bash
cargo run --release examples/fibonacci.ax
```

## Quick Reference

| Feature | Syntax |
|---------|--------|
| Variable | `let x = 10;` |
| Assignment | `x = 20;` |
| If-Else | `if (cond) { } else { }` |
| While | `while (cond) { }` |
| For | `for i in range(10) { }` |
| Function | `fn name(params) { }` |
| Call | `name(args);` |

## Next Steps

- [Language Reference](language-reference.md) - Full syntax guide
- [Examples](examples.md) - More code samples
