# Examples

## Hello World

```javascript
// hello.ax
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

## Function Definitions

```javascript
// Simple function with no parameters
fn greet() {
    print("Hello!");
}

// Function with one parameter
fn square(x) {
    x * x;
}

// Function with multiple parameters
fn add(a, b) {
    a + b;
}

// Function with local variables
fn calculateArea(width, height) {
    let area = width * height;
    area;
}

// Recursive function (fibonacci)
fn fib(n) {
    if (n <= 1) {
        n;
    } else {
        fib(n - 1) + fib(n - 2);
    }
}

// Calling defined functions
greet();
let result = square(5);
let sum = add(10, 20);
let fibValue = fib(10);
```

## Function Calls

```javascript
// No arguments
foo();

// Single argument
print(42);
print("Hello, World!");

// Multiple arguments
add(1, 2, 3);
greet("Alice", 25);

// Expression as argument
print(1 + 2 * 3);
calculate(x + y, z * 2);

// Nested function calls
print(add(1, 2));
max(min(a, b), c);

// Function call in expressions
let result = add(1, 2) * 3;
let doubled = multiply(getValue(), 2);

// Function call in assignment
x = compute(a, b);

// Function call with different literal types
check(true);
process(null);
format(42, 3.14, "text");
```

## While Loop

```javascript
// Basic while loop
let i = 0;
while (i < 5) {
    print(i);
    i = i + 1;
}

// Sum numbers 1 to 10
let sum = 0;
let n = 1;
while (n <= 10) {
    sum = sum + n;
    n = n + 1;
}
// sum is now 55

// Countdown
let count = 10;
while (count > 0) {
    print(count);
    count = count - 1;
}
print("Liftoff!");

// Find first power of 2 greater than 100
let power = 1;
while (power <= 100) {
    power = power * 2;
}
// power is now 128

// Nested while loops
let i = 0;
while (i < 3) {
    let j = 0;
    while (j < 3) {
        print(i * 3 + j);
        j = j + 1;
    }
    i = i + 1;
}
```

## For Loop

```javascript
// Basic for loop with range
for i in range(5) {
    print(i);  // 0, 1, 2, 3, 4
}

// For loop with start and end
for i in range(1, 10) {
    print(i);  // 1 to 9
}

// Sum using for loop
let sum = 0;
for n in range(1, 11) {
    sum = sum + n;
}
// sum is now 55

// Nested for loops (multiplication table)
for i in range(1, 4) {
    for j in range(1, 4) {
        print(i * j);
    }
}

// Factorial using for loop
let factorial = 1;
for i in range(1, 6) {
    factorial = factorial * i;
}
// factorial is now 120
```

## Combined Examples

```javascript
// Fibonacci with while loop
fn fibonacci(n) {
    let a = 0;
    let b = 1;
    let i = 0;
    while (i < n) {
        let temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }
    a;
}

// Prime check
fn isPrime(n) {
    if (n < 2) {
        false;
    } else {
        let i = 2;
        let result = true;
        while (i * i <= n) {
            if (n % i == 0) {
                result = false;
            }
            i = i + 1;
        }
        result;
    }
}

// Find all primes up to n
fn printPrimes(max) {
    for n in range(2, max) {
        if (isPrime(n)) {
            print(n);
        }
    }
}

// GCD using Euclidean algorithm
fn gcd(a, b) {
    while (b != 0) {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a;
}

// Using the functions
let fib10 = fibonacci(10);
let prime = isPrime(17);
printPrimes(20);
let divisor = gcd(48, 18);  // 6
```

## Running Examples

All examples are in the `examples/` folder:

```bash
cargo run --release examples/hello.ax
cargo run --release examples/fibonacci.ax
cargo run --release examples/builtins.ax
cargo run --release examples/scoping_explained.ax
```
