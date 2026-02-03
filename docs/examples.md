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

// Function with return
fn square(x) {
    return x * x;
}

// Function with multiple parameters
fn add(a, b) {
    return a + b;
}

// Function with local variables
fn calculateArea(width, height) {
    let area = width * height;
    return area;
}

// Recursive function (fibonacci) with return
fn fib(n) {
    if (n <= 1) {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

// Early return
fn abs(x) {
    if (x < 0) {
        return -x;
    }
    return x;
}

// Calling defined functions
greet();
let result = square(5);
let sum = add(10, 20);
let fibValue = fib(10);
let positive = abs(-42);  // 42
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

## Break and Continue

```javascript
// Break: exit a loop early
let n = 1;
while (true) {
    if (n * 7 > 50) {
        break;
    }
    n = n + 1;
}
// n is 8 (8 * 7 = 56)

// Break in a for loop
let result = 0;
for i in range(100) {
    if (i > 10) {
        break;
    }
    result = result + i;
}
// result is 55

// Continue: skip to the next iteration
let sum = 0;
for i in range(1, 11) {
    if (i % 2 == 0) {
        continue;  // skip even numbers
    }
    sum = sum + i;
}
// sum is 25 (1+3+5+7+9)

// Continue in a while loop
let i = 0;
let count = 0;
while (i < 10) {
    i = i + 1;
    if (i % 3 == 0) {
        continue;
    }
    count = count + 1;
}
// count is 7
```

## Return Statement

```javascript
// Early return from a function
fn abs(x) {
    if (x < 0) {
        return -x;
    }
    return x;
}

// Return in a loop inside a function
fn findFirst(list, target) {
    let i = 0;
    while (i < list.len()) {
        if (list.get(i) == target) {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

let idx = findFirst([10, 20, 30], 20);  // 1

// Functions without return yield null
fn doNothing() {
    let x = 1;
}

let result = doNothing();  // null
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
    return a;
}

// Prime check with early return
fn isPrime(n) {
    if (n < 2) {
        return false;
    }
    let i = 2;
    while (i * i <= n) {
        if (n % i == 0) {
            return false;
        }
        i = i + 1;
    }
    return true;
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
    return a;
}

// Sum with break
fn sumUntil(limit) {
    let sum = 0;
    let i = 1;
    while (true) {
        if (sum + i > limit) {
            break;
        }
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

// Using the functions
let fib10 = fibonacci(10);
let prime = isPrime(17);
printPrimes(20);
let divisor = gcd(48, 18);  // 6
let bounded = sumUntil(20);  // 15 (1+2+3+4+5)
```

## Classes

```javascript
// Define a class with constructor and methods
class Counter {
    fn init(self, start) {
        self.count = start;
    }

    fn increment(self) {
        self.count = self.count + 1;
        self.count;
    }

    fn get(self) {
        self.count;
    }
}

// Create instances
let c = new Counter(0);
c.increment();
c.increment();
c.increment();
print(c.get());  // 3

// Multiple independent instances
let a = new Counter(10);
let b = new Counter(20);
print(a.get());  // 10
print(b.get());  // 20
```

## Inheritance

```javascript
class Animal {
    let name = "";

    fn speak(self) {
        "sound";
    }
}

class Dog : Animal {
    fn bark(self) {
        "woof";
    }
}
```

## Static Access (`::`)

```javascript
// Class-level properties accessed via ::
class Config {
    let max_retries = 3;
    let timeout = 30;
}

print(Config::max_retries);  // 3
print(Config::timeout);      // 30

// Static methods (no self parameter) accessed via ::
class MathUtils {
    fn add(a, b) {
        a + b;
    }

    fn square(x) {
        x * x;
    }
}

print(MathUtils::add(10, 20));  // 30
print(MathUtils::square(5));    // 25

// Mixing static and instance access
class Counter {
    let default_start = 0;

    fn init(self, n) {
        self.count = n;
    }

    fn get(self) {
        self.count;
    }
}

let c = new Counter(5);
print(Counter::default_start);  // 0 (class-level)
print(c.get());                 // 5 (instance-level)

// Static access in expressions
class Factory {
    fn magic() {
        42;
    }
}

let result = Factory::magic() + 8;  // 50
```

## Lists

```javascript
// Create lists
let nums = [1, 2, 3, 4, 5];
let empty = [];

// List methods
print(nums.len());       // 5
print(nums.get(0));      // 1
print(nums.get(-1));     // 5

let more = nums.push(6);
let combined = [1, 2].concat([3, 4]);
```

## String Methods

```javascript
let greeting = "Hello";
print(greeting.len());                    // 5
print(greeting.concat(", World!"));       // "Hello, World!"
```

## Imports

```javascript
// -- math.ax (the module) --

fn add(a, b) {
    return a + b;
}

fn multiply(a, b) {
    return a * b;
}

let PI = 3;
```

```javascript
// -- main.ax (imports from math.ax) --

from math import add, multiply, PI;

// Use imported functions
let sum = add(10, 20);
print(sum);  // 30

let product = multiply(4, 5);
print(product);  // 20

// Use imported variable
print(PI);  // 3

// Combine imported functions
let result = add(multiply(3, 4), 5);
print(result);  // 17
```

```javascript
// Importing a class from another module

// -- shapes.ax --
class Circle {
    fn init(self, r) {
        self.radius = r;
    }

    fn area(self) {
        return self.radius * self.radius * 3;
    }
}

// -- app.ax --
from shapes import Circle;

let c = new Circle(5);
print(c.area());  // 75
```

## Running Examples

All examples are in the `examples/` folder:

```bash
cargo run --release examples/hello.ax
cargo run --release examples/fibonacci.ax
cargo run --release examples/builtins.ax
cargo run --release examples/classes.ax
cargo run --release examples/scoping_explained.ax
cargo run --release examples/imports.ax
```
