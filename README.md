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
- **Variables**: `let` for declaration and updates (auto-detects create vs update)
- **Control Flow**: `if` expressions, `while` loops, and `for` loops
- **Syntactic Sugar**: `++`/`--` for increment/decrement, `for` loops transform to `while`
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
- `examples/builtins.axe` - **All built-in functions with simple examples** (start here!)
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

Axe uses `let` for both creating and updating variables:

```lisp
; Creating variables
(let x 10)              ; Declare variable x = 10
x                       ; Get value: 10

; Updating variables
(let x 20)              ; Update x to 20 (if exists, update; if not, create)

; Scoping behavior
(let global 100)

(fn update_global ()
  (let global 999)      ; Updates global (variable exists in parent scope)
  global)               ; Returns 999

(update_global)         ; Returns 999
global                  ; Now 999 (was updated!)

; Local variables
(fn my_func ()
  (let local 42)        ; Creates new local variable
  local)                ; Returns 42
```

**How `let` works:**
- If variable exists in current or parent scope → **updates** it
- If variable doesn't exist → **creates** it in current scope

**Quick Reference:**
```lisp
(let x 10)              ; Declaring new variables
(let x 20)              ; Updating existing variables

(let counter 0)
(fn increment ()
  (let counter (+ counter 1)))  ; Updates global counter

(for (let i 0) (< i 10) (++ i)
  (print i))            ; i is created by for loop
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
(let counter 5)
(while (> counter 0)
    (let counter (- counter 1)))

; Multiple statements in while
(while (< i 10)
    (let sum (+ sum i))
    (let i (+ i 1)))
```

#### For Loops

For loops are syntactic sugar that transform to `while` loops:

```lisp
; Basic for loop
(for (let i 0) (< i 5) (++ i)
    (print i))          ; Prints 0 1 2 3 4

; For loop with decrement
(for (let count 10) (> count 0) (-- count)
    (print count))      ; Countdown from 10 to 1

; Nested for loops
(for (let i 1) (<= i 3) (++ i)
    (for (let j 1) (<= j 3) (++ j)
        (print i "*" j "=" (* i j))))

; For loop with arrays
(let numbers (list 1 2 3 4 5))
(for (let i 0) (< i (len numbers)) (++ i)
    (print (get numbers i)))
```

#### Increment/Decrement

```lisp
; Increment
(let x 0)
(++ x)                  ; x becomes 1

; Decrement
(let y 10)
(-- y)                  ; y becomes 9

; In loops
(for (let i 0) (< i 5) (++ i)
    (print i))
```

### Blocks

Blocks create new scopes:

```lisp
(begin
    (let x 10)
    (let y 20)
    (+ x y))        ; Returns 30
```

### Functions

Define and call functions (functions are syntactic sugar for lambdas):

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

; Higher-order function (nested functions)
(fn makeAdder (x)
    (fn adder (y) (+ x y)))
(let add5 (makeAdder 5))
(add5 10)           ; Returns 15

; Lambda expressions (what fn desugars to)
(let add (lambda (a b) (+ a b)))
(add 5 3)           ; Returns 8
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

### Complete Examples

#### Sum from 1 to 5 (traditional)
```lisp
(let sum 0)
(let i 1)
(while (<= i 5)
    (let sum (+ sum i))
    (let i (+ i 1)))
sum                 ; Returns 15
```

#### Sum from 1 to 5 (using for loop)
```lisp
(let sum 0)
(for (let i 1) (<= i 5) (++ i)
    (let sum (+ sum i)))
sum                 ; Returns 15
```

#### Fibonacci with for loop
```lisp
(let a 0)
(let b 1)
(for (let i 0) (< i 10) (++ i)
    (print a)
    (let temp a)
    (let a b)
    (let b (+ temp b)))
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
- `Set(name, expr)` - Variable declaration/update (`let`)
- `Var(name)` - Variable reference
- `List(elements)` - List literal
- `Block(exprs)` - Block with new scope
- `If(condition, then, else)` - Conditional
- `While(condition, body)` - Loop
- `Lambda(params, body)` - Lambda expression (anonymous function)
- `Function(name, params, body)` - Function definition (syntactic sugar for `let` + `lambda`)
- `FunctionCall(name, args)` - Function call
- `Inc(name)` - Increment (syntactic sugar: `i++` → `let i (+ i 1)`)
- `Dec(name)` - Decrement (syntactic sugar: `i--` → `let i (- i 1)`)
- `For(init, cond, update, body)` - For loop (syntactic sugar for `while`)

## Truthiness Rules

Falsy values:
- `null`
- `false`
- `0` (integer zero)
- `0.0` (float zero)

Everything else is truthy (including empty strings).

## Scoping Rules

Axe implements lexical scoping with smart variable binding:

### How `let` Works

`let` intelligently decides whether to create or update:

```lisp
(let x 10)          ; Creates x (doesn't exist)
(let x 20)          ; Updates x (exists in current scope)

(fn test ()
    (let x 100))    ; Updates x (exists in parent scope)

(test)
x                   ; Now 100 (was updated!)
```

### Key Scoping Rules

1. **Functions create new scopes** - Each function call has its own environment
2. **`let` searches parent scopes** - If variable exists anywhere, it updates it; otherwise creates it
3. **Variable lookup** searches current scope, then parent scopes recursively
4. **Closures capture** their defining environment
5. **Blocks, `while`, and `if` do NOT create new scopes**

### Examples

#### Updating parent scope from function:
```lisp
(let counter 0)

(fn increment ()
    (let counter (+ counter 1)))  ; Updates global counter

(increment)
counter             ; Now 1

(increment)  
counter             ; Now 2
```

#### Local variable in same scope as loop:
```lisp
(let x 10)
(while (> x 0)
    (let x (- x 1)))  ; Updates x in same scope
x                     ; Now 0
```

#### New variable in function:
```lisp
(fn create_local ()
    (let y 42))       ; Creates new local y

(create_local)
y                     ; Error: undefined variable
```

