# Axe Programming Language

A lightweight programming language interpreter written in Rust, featuring C-like syntax with variables, control flow, and block scoping.

## Features

- **C-like Syntax**: Familiar syntax with semicolons and braces
- **File Execution**: Run `.axe` files directly
- **Data Types**: Integers, floats, strings, booleans, null
- **Arithmetic Operations**: `+`, `-`, `*`, `/`, `%`
- **Comparison Operations**: `>`, `<`, `>=`, `<=`, `==`, `!=`
- **Logical Operations**: `&&`, `||`
- **Bitwise Operations**: `&`, `|`
- **Variables**: `let` for declaration, `=` for assignment
- **Control Flow**: `if`/`else` statements
- **Block Scoping**: Lexical scoping with `{}` blocks
- **Comments**: Single-line (`//`) and multi-line (`/* */`)

## Installation

```bash
cargo build --release
```

## Usage

### Running Files

Execute an `.axe` file:

```bash
cargo run --release <filename>.axe
# or after building:
./target/release/axe <filename>.axe
```

Example file (`hello.axe`):
```javascript
// This is a comment
let message = "Hello, World!";
```

Run it:
```bash
cargo run --release examples/hello.axe
```

**Examples:**
- `examples/hello.axe` - Hello world
- `examples/fibonacci.axe` - Arithmetic sequences
- `examples/simple_counter.axe` - Variable assignment
- `examples/builtins.axe` - Operators demo
- `examples/scoping_explained.axe` - Block scoping guide

## Language Syntax

### Comments

```javascript
// This is a single-line comment
let x = 10;  // Comments can go after code

/* This is a
   multi-line comment */
```

### Literals

```javascript
42                  // Integer
3.14                // Float
"hello"             // String
true                // Boolean true
false               // Boolean false
null                // Null value
```

### Arithmetic

```javascript
let a = 1 + 2;      // 3
let b = 5 - 3;      // 2
let c = 4 * 5;      // 20
let d = 10 / 2;     // 5
let e = 10 % 3;     // 1
let f = (2 + 3) * 4;  // 20 (parentheses for grouping)
```

### Comparisons (in if conditions)

Comparison operators are used in `if` conditions:

```javascript
if (5 > 3) {
    // executes when true
}

if (x >= 10) {
    // greater than or equal
}

if (a == b) {
    // equality check
}

if (a != b) {
    // inequality check
}
```

### Logical Operators

```javascript
true && true        // true
true && false       // false
false || true       // true
false || false      // false
```

### Bitwise Operators

```javascript
5 & 3               // 1  (binary AND)
5 | 3               // 7  (binary OR)
```

### Variables

```javascript
// Declaration with initialization
let x = 10;
let name = "Axe";
let active = true;

// Declaration without initialization (defaults to null)
let y;

// Multiple declarations
let a = 1, b = 2, c = 3;

// Assignment (variable must exist)
x = 20;
x = x + 10;
```

### Control Flow

#### If Statements

```javascript
if (x > 0) {
    let result = "positive";
}

if (x == 0) {
    let result = "zero";
} else {
    let result = "not zero";
}
```

### Blocks

Blocks create new scopes:

```javascript
{
    let x = 10;
    let y = 20;
    let sum = x + y;
}
// x, y, sum not accessible here
```

### Complete Examples

#### Conditional Logic
```javascript
let score = 85;
let grade;

if (score >= 90) {
    grade = "A";
} else {
    if (score >= 80) {
        grade = "B";
    } else {
        grade = "C";
    }
}
```

#### Complex Expressions
```javascript
let a = 10;
let b = 5;
let c = 3;

let result = (a + b) * c - (a / b);

// Comparisons in conditions
if (a > b) {
    let larger = a;
}
```

## Data Types

- `Int` - 64-bit integers
- `Float` - 64-bit floating point
- `Str` - Strings
- `Bool` - `true` or `false`
- `Null` - Null value

## Truthiness Rules

Falsy values:
- `null`
- `false`
- `0` (integer zero)
- `0.0` (float zero)

Everything else is truthy (including empty strings).

## Scoping Rules

Axe implements lexical scoping:

1. **Blocks create scopes** - Variables declared in a block are local to that block
2. **Inner scopes can access outer** - Child blocks can read/write parent variables
3. **`let` creates new variables** - Always creates in current scope
4. **`=` updates existing variables** - Must reference a declared variable

```javascript
let x = 10;

{
    let y = 20;     // Local to this block
    x = 15;         // Updates outer x
}

// x is 15, y is not accessible
```

## Roadmap

Features planned for future versions:
- While loops
- For loops
- Functions
- Lists/Arrays
- Built-in functions (print, len, etc.)
- Classes
