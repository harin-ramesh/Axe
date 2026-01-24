# Examples

## Hello World

```javascript
// hello.axe
let message = "Hello, World!";
```

## Arithmetic

```javascript
// Basic operations
let a = 10 + 5;   // 15
let b = 10 - 5;   // 5
let c = 10 * 5;   // 50
let d = 10 / 5;   // 2
let e = 10 % 3;   // 1

// Complex expressions
let result = (10 + 5) * 2;       // 30
let nested = ((1 + 2) * 3) + 4;  // 13

// Unary minus
let negative = -42;
let diff = 10 - -5;  // 15
```

## Fibonacci Sequence

```javascript
// Manual fibonacci steps (without loops)
let a = 0;
let b = 1;

let temp = a + b;  // 1
a = b;
b = temp;

temp = a + b;      // 2
a = b;
b = temp;

temp = a + b;      // 3
a = b;
b = temp;

temp = a + b;      // 5
a = b;
b = temp;

temp = a + b;      // 8
a = b;
b = temp;

// b now contains fib(7) = 8
```

## Operators Demo

```javascript
// Arithmetic
let a = 10 + 5;
let b = 10 - 5;
let c = 10 * 5;
let d = 10 / 5;
let e = 10 % 3;

// Logical
let and_result = true && true;
let or_result = false || true;

// Bitwise
let band = 5 & 3;  // 1
let bor = 5 | 3;   // 7

// Comparisons in conditions
if (10 > 5) {
    let x = 1;
}

if (5 == 5) {
    let y = 2;
}
```

## Block Scoping

```javascript
let x = 10;

// Blocks create new scopes
{
    let y = 20;
    let z = x + y;  // Can access outer x
}

// If statements with blocks
if (x > 5) {
    let result = x * 2;
} else {
    let result = x / 2;
}

// Nested blocks
{
    let outer = 1;
    {
        let inner = 2;
        let sum = outer + inner;
    }
}
```

## Conditional Logic

```javascript
let score = 85;
let grade;

if (score >= 90) {
    grade = "A";
} else {
    if (score >= 80) {
        grade = "B";
    } else {
        if (score >= 70) {
            grade = "C";
        } else {
            grade = "F";
        }
    }
}
```

## Variable Shadowing

```javascript
let x = 10;

{
    let x = 20;     // New x, shadows outer
    // x is 20 here
}

// x is still 10 here
```

## Running Examples

All examples are in the `examples/` folder:

```bash
cargo run --release examples/hello.axe
cargo run --release examples/fibonacci.axe
cargo run --release examples/builtins.axe
cargo run --release examples/scoping_explained.axe
```
