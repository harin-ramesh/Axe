# Axe Programming Language

A lightweight programming language interpreter written in Rust, featuring C-like syntax with variables, control flow, and block scoping.

## Features

- C-like syntax with semicolons and braces
- File execution for `.axe` files
- Data types: integers, floats, strings, booleans, null
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparisons: `>`, `<`, `>=`, `<=`, `==`, `!=`
- Logical: `&&`, `||`
- Bitwise: `&`, `|`
- Variables with `let` and assignment with `=`
- Control flow with `if`/`else`
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
let x = 10;
let y = 20;

if (x < y) {
    let sum = x + y;
}
```

## Roadmap

- While loops
- For loops
- Functions
- Lists/Arrays
- Built-in functions (print, len, etc.)
- Classes
