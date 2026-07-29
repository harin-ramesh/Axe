# Axe Programming Language

A lightweight programming language written in Rust, with a C-like syntax and support for functions, closures, and classes. Axe compiles to bytecode and runs on a stack-based virtual machine with a mark-sweep garbage collector.

## Quick Start

Create a file called `hello.ax`:

```javascript
println("Hello, World!");
```

Then build and run it:

```bash
# Build
cargo build --release

# Run your program
cargo run --release hello.ax
```

You should see:

```
Hello, World!
```

To start the interactive REPL instead:

```bash
cargo run --release
```

## Local Setup

### Prerequisites

**Rust** (1.85+ recommended) - Install via [rustup](https://rustup.rs/):

### Installation

```bash
# Clone the repository
git clone git@github.com:harin-ramesh/Axe.git axe
cd axe

# Build the project
cargo build --release
```

### Usage

```bash
# Run a script file
cargo run --release examples/hello.ax

# Or use the compiled binary directly
./target/release/axe examples/hello.ax

# Start the interactive REPL
cargo run --release

# Print the bytecode disassembly of a file (no execution)
./target/release/axe --disassemble examples/hello.ax
```

## Architecture

Source code flows through a tokeniser, recursive-descent parser, and bytecode compiler (with constant folding), then executes on a stack-based VM:

- **Bytecode VM** — compact single-byte opcodes, call frames, closures via upvalues
- **Garbage collector** — non-moving mark-sweep with slot reuse; heap stays proportional to live data (run with `AXE_GC_STRESS=1` to collect at every safepoint for debugging)
- **Error reporting** — compile and runtime errors carry source line numbers, and runtime errors include a stack trace:

```
runtime error [line 2]: undefined property 'missing_field'
  in get_it (called from line 5)
  in process (called from line 11)
```

Exit codes follow convention: `65` for parse/compile errors, `70` for runtime errors.

### Running Tests

```bash
# Run all tests
cargo test

# Run parser + end-to-end tests only
cargo test --test parser

# Track VM performance
cargo run --release --bin bench   # quick µs/iter report
cargo bench                       # criterion, with regression tracking
```

## Features

- **C-like syntax** with semicolons and braces
- **Data types**: integers (i64), floats (f64), strings, booleans, null, lists
- **Variables** with block scoping and shadowing
- **Control flow**: if/else statements, while loops, for loops over ranges/lists
- **Functions** with `return`, recursion, and closures (captured variables outlive their frame)
- **Classes** with inheritance, instance methods (`.`), and static access (`::`)
- **Built-in functions**: `print`, `println`, `range`, `len`
- **Operators**: arithmetic, comparison, logical, and bitwise
- **Safety**: checked integer arithmetic, division-by-zero errors, call-depth limit — bad programs report errors, they don't crash the host

### Not yet supported

These parse (or are planned) but currently report a clean compile error:

- `break` / `continue` in loops
- `from module import ...;`
- Lambda expressions
- Index syntax `list[i]` and methods on strings/lists (`.len()`, `.concat()`, ...) — use the `len(x)` builtin and `for` loops meanwhile

## Examples

### Hello World
```javascript
println("Hello, World!");
```

### Variables and Expressions
```javascript
let x = 10;
let y = 20;
let sum = x + y;
println(sum);  // 30

// Multiple declarations
let a = 1, b = 2, c = 3;
```

### Functions
```javascript
// Define a function with explicit return
fn factorial(n) {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

println(factorial(5));  // 120

// Sum of squares using a for loop
fn sumOfSquares(limit) {
    let total = 0;
    for i in range(1, limit + 1) {
        total = total + i * i;
    }
    return total;
}

println(sumOfSquares(5));  // 55
```

### Closures
```javascript
fn counter() {
    let c = 0;
    fn inc() {
        c = c + 1;
        return c;
    }
    return inc;
}

let next = counter();
next();           // 1
next();           // 2
println(next());  // 3
```

### Control Flow
```javascript
// If-else
if (x > 0) {
    println("positive");
} else {
    println("non-positive");
}

// While loop
let i = 0;
while (i < 5) {
    println(i);
    i = i + 1;
}

// For loop over a range or list
for n in range(10) {
    println(n);  // 0 .. 9
}
for item in [10, 20, 30] {
    println(item);
}
```

### Classes
```javascript
class Counter {
    let default_start = 0;

    fn init(self, start) {
        self.count = start;
    }

    fn increment(self) {
        self.count = self.count + 1;
        return self.count;
    }

    fn get(self) {
        return self.count;
    }
}

let c = new Counter(0);
c.increment();
c.increment();
println(c.get());  // 2
```

### Static Access (`::`)
```javascript
// Access class-level properties and static methods with ::
class MathUtils {
    let PI = 3;

    fn add(a, b) {
        return a + b;
    }
}

MathUtils::PI;           // 3 (class-level property)
MathUtils::add(10, 20);  // 30 (static method, no self)

// Instance methods use . as before
class Box {
    fn init(self, v) {
        self.value = v;
    }
    fn get(self) {
        return self.value;
    }
}

let b = new Box(42);
b.get();  // 42 (instance method)
```

### Lists
```javascript
let numbers = [1, 2, 3, 4, 5];
println(len(numbers));   // 5

let total = 0;
for n in numbers {
    total = total + n;
}
println(total);          // 15
```

## Documentation

See the [docs](docs/index.md) folder for full documentation:

- [Getting Started](docs/getting-started.md)
- [Language Reference](docs/language-reference.md)
- [Examples](docs/examples.md)
- [VM Memory & GC](docs/vm-memory.md)

## Example Files

The `examples/` directory contains several example programs:

| File | Description |
|------|-------------|
| `hello.ax` | Simple hello world |
| `functions.ax` | Function definitions and usage |
| `loops.ax` | While and for loop examples |
| `fibonacci.ax` | Fibonacci-style arithmetic and assignment |
| `classes.ax` | Object-oriented programming examples |
| `builtins.ax` | Built-in operators and expressions |
| `scoping_explained.ax` | Variable scoping demonstration |
| `simple_counter.ax` | Variable declaration and assignment |
| `math.ax` | Utility module (for future import support) |

Run any example with:
```bash
cargo run --release examples/<filename>.ax
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
| Access | `.` (instance), `::` (static/class) |

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print(values...)` | Print values to stdout |
| `println(values...)` | Print values followed by a newline |
| `range(end)` | Generate list [0, 1, ..., end-1] |
| `range(start, end)` | Generate list [start, ..., end-1] |
| `len(x)` | Length of a list or string |
