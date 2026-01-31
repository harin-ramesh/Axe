# Axe Programming Language

A lightweight programming language interpreter written in Rust, featuring C-like syntax with variables, control flow, functions, and block scoping.

## Features

- C-like syntax with semicolons and braces
- File execution for `.ax` files
- Data types: integers, floats, strings, booleans, null, lists
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparisons: `>`, `<`, `>=`, `<=`, `==`, `!=`
- Logical: `&&`, `||`
- Bitwise: `&`, `|`
- Unary operators: `-`, `!`, `~`
- Variables with `let` and assignment with `=`
- Control flow with `if`/`else`
- Loops: `while` and `for` with `range()`
- Functions with `fn` keyword and recursion
- Classes with inheritance (`:`) and constructors (`init`)
- Instance access with `.` for properties and methods
- Static access with `::` for class-level properties and methods
- Methods on strings (`.len()`, `.concat()`) and lists (`.len()`, `.get()`, `.push()`, `.concat()`)
- Built-in functions: `print`, `type`, `range`
- Block scoping with `{}`
- Comments: `//` and `/* */`

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](getting-started.md) | Installation and first steps |
| [Language Reference](language-reference.md) | Complete syntax guide |
| [Examples](examples.md) | Code examples and tutorials |

## Quick Example

```javascript
// Function definition
fn factorial(n) {
    let result = 1;
    let i = 1;
    while (i <= n) {
        result = result * i;
        i = i + 1;
    }
    result;
}

// Using loops and functions
let sum = 0;
for i in range(1, 11) {
    sum = sum + i;
}

let fact5 = factorial(5);  // 120
```

## Roadmap

- Return statement
- Break/continue for loops
- Module system with imports
- Pattern matching
