# 🪓 Axe Programming Language

A lightweight S-expression based programming language interpreter written in Rust, featuring functions, variables, control flow, and proper scoping.

## Features

- **S-Expression Syntax**: Clean Lisp-like syntax with comments (`;`)
- **File Execution**: Run `.axe` files directly
- **Functions**: First-class functions with closures and recursion support
- **Data Types**: Integers, floats, strings, booleans, null, and lists
- **Lists**: First-class list data structure with built-in operations
- **Arithmetic Operations**: `+`, `-`, `*`, `/` for integers and floats
- **Comparison Operations**: `>`, `<`, `>=`, `<=`, `==`, `!=`
- **Variables**: `let` for declaration/shadowing, `assign` for updating existing variables
- **Control Flow**: `if` expressions and `while` loops
- **Block Scoping**: Lexical scoping with blocks
- **Boolean Type**: `true` and `false` with proper truthiness rules
- **Built-in Functions**: `print`, `len`, `get`, `push`, `type`, `concat`, `range`
- **Type Safety**: Strong type checking at runtime
- **Interactive REPL**: Command-line interface for experimentation with multi-line support

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
```lisp
; This is a comment
(print "Hello, World!")

(fn greet (name)
  (print "Hello," name))

(greet "Axe")
```

Run it:
```bash
cargo run --release examples/hello.axe
```

**More Examples:**
- `examples/hello.axe` - Hello world with functions
- `examples/fibonacci.axe` - Recursive fibonacci
- `examples/list_demo.axe` - Comprehensive list operations
- `examples/assign_demo.axe` - Real-world assign patterns
- `examples/scoping_explained.axe` - **Complete scoping guide** (highly recommended!)

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

axe> (let x 5)
=> Int(5)

axe> (* x 2)
=> Int(10)

axe> (if (> x 3) "big" "small")
=> Str("big")

axe> (fn add (a b) (+ a b))
=> <function(a, b)>

axe> (add 10 20)
=> 30

axe> (print "Hello, World!")
Hello, World!
=> null

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

### Comments

```lisp
; This is a single-line comment
(print "Hello")  ; Comments can go after code
```

### Literals

```lisp
42                  ; Integer
3.14                ; Float
"hello"             ; String
true                ; Boolean true
false               ; Boolean false
null                ; Null value
(list 1 2 3)        ; List
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

Axe provides two keywords for working with variables:

- **`let`** - Creates a new variable or shadows an existing one in the current scope
- **`assign`** - Updates an existing variable (searches parent scopes)

```lisp
; Creating variables
(let x 10)              ; Declare variable x = 10
x                       ; Get value: 10

; Updating in same scope
(let x 20)              ; Shadows x in current scope (creates new binding)
(assign x 30)           ; Updates the existing x

; Scoping behavior
(let global 100)

(fn my_func ()
  (let global 999)      ; Creates NEW local variable (shadows)
  global)               ; Returns 999

(my_func)               ; Returns 999
global                  ; Still 100 (unchanged)

(fn update_global ()
  (assign global 999)   ; Updates the ACTUAL global variable
  global)               ; Returns 999

(update_global)         ; Returns 999
global                  ; Now 999 (was updated!)
```

**Key Difference:**
- `let` in a function creates a **new local variable** (shadowing)
- `assign` **updates the existing variable** wherever it was defined

**Quick Reference:**
```lisp
; When to use let:
(let x 10)              ; Declaring new variables
(fn my_func ()
  (let result 42)       ; Local variables in functions
  result)

; When to use assign:
(let counter 0)
(fn increment ()
  (assign counter (+ counter 1)))  ; Updating global state

(let i 0)
(while (< i 10)
  (assign i (+ i 1)))   ; Updating loop variables
```

**👉 For a complete guide with examples, see `examples/scoping_explained.axe`**

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
(let counter 5)
(while (> counter 0)
    (let counter (- counter 1)))

; Multiple statements in while
(while (< i 10)
    (let sum (+ sum i))
    (let i (+ i 1)))
```

### Blocks

Blocks create new scopes:

```lisp
(block
    (let x 10)
    (let y 20)
    (+ x y))        ; Returns 30
```

### Functions

Define and call functions:

```lisp
; Simple function
(fn add (a b) (+ a b))
(add 5 3)           ; Returns 8

; Recursive function
(fn factorial (n)
    (if (<= n 1)
        1
        (* n (factorial (- n 1)))))
(factorial 5)       ; Returns 120

; Function with closure
(let x 10)
(fn addX (y) (+ x y))
(addX 5)            ; Returns 15

; Higher-order function
(fn makeAdder (x)
    (fn adder (y) (+ x y)))
(let add5 (makeAdder 5))
(add5 10)           ; Returns 15
```

### Lists

Create and manipulate lists:

```lisp
; Create a list
(let numbers (list 1 2 3 4 5))

; Get length
(len numbers)                ; Returns 5

; Get element by index (supports negative indices)
(get numbers 0)              ; Returns 1
(get numbers -1)             ; Returns 5 (last element)

; Push element (returns new list)
(let extended (push numbers 6))
(print extended)             ; Prints: [1, 2, 3, 4, 5, 6]

; Concatenate lists
(concat (list 1 2) (list 3 4))  ; Returns [1, 2, 3, 4]

; Range functions
(range 5)                    ; Returns [0, 1, 2, 3, 4]
(range 2 7)                  ; Returns [2, 3, 4, 5, 6]
(range 0 10 2)               ; Returns [0, 2, 4, 6, 8]
(range 10 0 -2)              ; Returns [10, 8, 6, 4, 2]

; Nested lists
(let matrix (list (list 1 2) (list 3 4)))
(get (get matrix 0) 1)       ; Returns 2

; Mixed type lists
(let mixed (list 1 "hello" true null 3.14))
```

### Built-in Functions

#### print
Prints values to stdout:
```lisp
(print "Hello" "World" 42)   ; Prints: "Hello" "World" 42
```

#### len
Returns the length of a list or string:
```lisp
(len (list 1 2 3))           ; Returns 3
(len "hello")                ; Returns 5
```

#### get
Gets an element from a list by index (supports negative indices):
```lisp
(get (list 10 20 30) 1)      ; Returns 20
(get (list 10 20 30) -1)     ; Returns 30
```

#### push
Adds an element to a list (returns new list):
```lisp
(push (list 1 2) 3)          ; Returns [1, 2, 3]
```

#### type
Returns the type of a value as a string:
```lisp
(type 42)                    ; Returns "int"
(type "hello")               ; Returns "string"
(type (list 1 2))            ; Returns "list"
```

#### concat
Concatenates strings or lists:
```lisp
(concat "Hello" " " "World") ; Returns "Hello World"
(concat (list 1 2) (list 3)) ; Returns [1, 2, 3]
```

#### range
Generates a list of integers:
```lisp
(range 5)                    ; Returns [0, 1, 2, 3, 4]
(range 2 7)                  ; Returns [2, 3, 4, 5, 6]
(range 0 10 2)               ; Returns [0, 2, 4, 6, 8]
```

### Complete Example: Sum from 1 to 5

```lisp
; Using assign to update variables
(let sum 0)
(let i 1)
(while (<= i 5)
    (assign sum (+ sum i))
    (assign i (+ i 1)))
sum                 ; Returns 15
```

## Expression Types

### Literals
- `Null` - Null value
- `Bool(bool)` - Boolean
- `Int(i64)` - Integer  
- `Float(f64)` - Float
- `Str(String)` - String
- `List(Vec<Value>)` - List

### Operations
- `Binary(op, left, right)` - Binary operations
- `Set(name, expr)` - Variable declaration/shadowing (`let`)
- `Assign(name, expr)` - Variable update (`assign`)
- `Var(name)` - Variable reference
- `List(elements)` - List literal
- `Block(exprs)` - Block with new scope
- `If(condition, then, else)` - Conditional
- `While(condition, body)` - Loop
- `Function(name, params, body)` - Function definition
- `FunctionCall(name, args)` - Function call

## Truthiness Rules

Falsy values:
- `null`
- `false`
- `0` (integer zero)
- `0.0` (float zero)

Everything else is truthy (including empty strings).

## Scoping Rules

Axe implements lexical scoping with two important keywords:

### Variable Shadowing with `let`
`let` creates a **new variable** in the current scope, even if one exists in a parent scope:

```lisp
(let x 10)
(fn my_func ()
    (let x 100)     ; Creates NEW local x (shadows global)
    x)              ; Returns 100

(my_func)           ; Returns 100
x                   ; Still 10 (global unchanged)
```

### Variable Updating with `assign`
`assign` **updates an existing variable** by searching up the scope chain:

```lisp
(let counter 0)

(fn increment ()
    (assign counter (+ counter 1)))  ; Updates global counter

(increment)
counter             ; Now 1

(increment)
counter             ; Now 2
```

### Key Scoping Rules

1. **Only functions create new scopes** - `block`, `while`, and `if` do NOT create scopes
2. **`let` always creates/shadows** in the current scope
3. **`assign` always updates** the existing variable (searches parent scopes)
4. **Variable lookup** searches current scope, then parent scopes
5. **Closures capture** their defining environment

### Important Note
Since only functions create scopes, this works:

```lisp
(let x 10)
(while (> x 0)
    (assign x (- x 1)))  ; Updates x in same scope
x                         ; Now 0
```

But within a function, `let` creates a shadow:

```lisp
(let x 10)
(fn test ()
    (let x 20))     ; New local variable
(test)
x                   ; Still 10
```

## REPL Commands

- `help` - Show help message
- `exit` or `quit` - Exit the REPL
- Any Axe expression - Evaluate and print result

## Error Handling

Descriptive error messages:
- `"undefined variable"` - Variable not found
- `"undefined function"` - Function not found
- `"invalid variable name"` - Invalid identifier
- `"invalid function name"` - Invalid function name
- `"invalid parameter name"` - Invalid parameter name
- `"argument count mismatch"` - Wrong number of arguments
- `"division by zero"` - Division by zero
- `"type error"` - Type mismatch
- `"not a function"` - Attempted to call a non-function value
- `"Unterminated string"` - Parse error
- `"Expected operator"` - Syntax error

