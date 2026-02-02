# Language Reference

## Comments

```javascript
// Single-line comment
let x = 10;  // Inline comment

/* Multi-line
   comment */
```

## Data Types

| Type | Description | Example |
|------|-------------|---------|
| Int | 64-bit integer | `42`, `-17` |
| Float | 64-bit float | `3.14`, `-0.5` |
| Str | String | `"hello"` |
| Bool | Boolean | `true`, `false` |
| Null | Null value | `null` |

## Literals

```javascript
42          // Integer
3.14        // Float
"hello"     // String
true        // Boolean
false       // Boolean
null        // Null
```

## Variables

### Declaration

```javascript
let x = 10;
let name = "Axe";
let active = true;
let y;                  // defaults to null
let a = 1, b = 2, c = 3;  // multiple
```

### Assignment

```javascript
let x = 10;
x = 20;
x = x + 10;
```

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `1 + 2` = `3` |
| `-` | Subtraction | `5 - 3` = `2` |
| `*` | Multiplication | `4 * 5` = `20` |
| `/` | Division | `10 / 2` = `5` |
| `%` | Modulo | `10 % 3` = `1` |

```javascript
let result = (2 + 3) * 4;  // 20
let negative = -42;
```

### Comparison

| Operator | Description |
|----------|-------------|
| `>` | Greater than |
| `<` | Less than |
| `>=` | Greater or equal |
| `<=` | Less or equal |
| `==` | Equal |
| `!=` | Not equal |

### Logical

| Operator | Description |
|----------|-------------|
| `&&` | AND |
| `\|\|` | OR |
| `!` | NOT |

```javascript
true && false   // false
true || false   // true
!true           // false
!false          // true
```

### Bitwise

| Operator | Description |
|----------|-------------|
| `&` | Bitwise AND |
| `\|` | Bitwise OR |
| `~` | Bitwise NOT |

```javascript
5 & 3   // 1
5 | 3   // 7
~5      // -6
```

### Unary

| Operator | Description |
|----------|-------------|
| `-` | Negation |
| `+` | Positive (no-op) |
| `!` | Logical NOT |
| `~` | Bitwise NOT |

```javascript
let neg = -42;
let pos = +42;
let not = !true;    // false
let inv = ~5;       // -6
```

## Control Flow

### If Statement

```javascript
if (x > 0) {
    let result = "positive";
}
```

### If-Else

```javascript
if (x == 0) {
    let result = "zero";
} else {
    let result = "not zero";
}
```

### Nested If-Else

```javascript
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

### While Loop

The `while` loop repeats a block while a condition is true:

```javascript
let i = 0;
while (i < 5) {
    // body executes while i < 5
    i = i + 1;
}
```

#### Examples

```javascript
// Sum 1 to 10
let sum = 0;
let n = 1;
while (n <= 10) {
    sum = sum + n;
    n = n + 1;
}
// sum is 55

// Countdown
let count = 10;
while (count > 0) {
    count = count - 1;
}
```

### For Loop

The `for` loop iterates over a range:

```javascript
for variable in range(end) { ... }
for variable in range(start, end) { ... }
```

#### Examples

```javascript
// Iterate 0 to 4
for i in range(5) {
    // i = 0, 1, 2, 3, 4
}

// Iterate 1 to 9
for i in range(1, 10) {
    // i = 1, 2, ..., 9
}

// Sum with for loop
let sum = 0;
for n in range(1, 11) {
    sum = sum + n;
}
// sum is 55

// Nested for loops
for i in range(3) {
    for j in range(3) {
        // executes 9 times
    }
}
```

### Break

The `break` statement exits the innermost enclosing `while` or `for` loop immediately:

```javascript
// Find first multiple of 7 greater than 50
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
// result is 55 (0+1+2+...+10)
```

### Continue

The `continue` statement skips the rest of the current iteration and moves to the next one:

```javascript
// Sum only odd numbers from 1 to 10
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
        continue;  // skip multiples of 3
    }
    count = count + 1;
}
// count is 7
```

## Functions

### Function Definition

Define functions using the `fn` keyword:

```javascript
fn functionName(param1, param2) {
    // function body
    return result;
}
```

#### Examples

```javascript
// No parameters
fn greet() {
    print("Hello!");
}

// Single parameter with return
fn square(x) {
    return x * x;
}

// Multiple parameters
fn add(a, b) {
    return a + b;
}

// With local variables
fn calculateArea(width, height) {
    let area = width * height;
    return area;
}
```

### Return Statement

Use `return` to explicitly return a value from a function:

```javascript
fn abs(x) {
    if (x < 0) {
        return -x;
    } else {
        return x;
    }
}

// Early return
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
```

If a function completes without hitting a `return` statement, it returns `null`.

### Function Calls

Call functions by name with arguments in parentheses:

```javascript
// No arguments
greet();

// With arguments
let result = square(5);      // 25
let sum = add(10, 20);       // 30

// Expression as argument
print(1 + 2 * 3);

// Nested calls
print(add(1, 2));

// In expressions
let doubled = square(3) * 2;  // 18
```

### Recursion

Functions can call themselves:

```javascript
fn factorial(n) {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fn fibonacci(n) {
    if (n <= 1) {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

let fact5 = factorial(5);  // 120
let fib10 = fibonacci(10); // 55
```

## Blocks and Scoping

Blocks create new scopes with `{}`:

```javascript
let x = 10;

{
    let y = 20;     // local to this block
    x = 15;         // updates outer x
}

// x is 15, y is not accessible
```

### Scoping Rules

1. Blocks create scopes - variables are local to their block
2. Inner scopes access outer - child blocks can read/write parent variables
3. `let` creates new variables - always in current scope
4. `=` updates existing - must reference declared variable

### Shadowing

```javascript
let x = 10;
{
    let x = 20;  // shadows outer x
}
// x is still 10
```

## Classes

### Class Definition

Define classes using the `class` keyword. Properties are declared with `let`, methods with `fn`:

```javascript
class ClassName {
    let property = value;

    fn init(self, params) {
        self.property = params;
    }

    fn method(self) {
        self.property;
    }
}
```

- Methods that operate on instances take `self` as the first parameter
- `init` is the constructor, called automatically by `new`
- Properties declared with `let` are class-level (static) properties

### Object Instantiation

Create instances using the `new` keyword:

```javascript
let obj = new ClassName(args);
```

### Instance Access (`.`)

Use `.` to access properties and methods on instances:

```javascript
let p = new Point(10, 20);
p.x;            // access instance property
p.distance();   // call instance method
```

### Static Access (`::`)

Use `::` to access class-level properties and static methods directly on the class, without creating an instance:

```javascript
class Config {
    let max_retries = 3;
    let timeout = 30;

    fn default_name() {
        "unnamed";
    }
}

Config::max_retries;     // 3
Config::timeout;         // 30
Config::default_name();  // "unnamed"
```

Static methods are methods defined **without** `self` as the first parameter. They belong to the class itself, not to instances.

Instance methods (with `self`) are only accessible via `.` on an instance. Static properties and methods (without `self`) are accessible via `::` on the class.

| Access | Syntax | What it accesses |
|--------|--------|-----------------|
| `.` | `instance.prop` | Instance properties and methods (with `self`) |
| `::` | `Class::prop` | Class-level properties and static methods (without `self`) |

### Inheritance

Use `:` to inherit from a parent class:

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

### Chaining

Instance method calls can be chained with `.`:

```javascript
obj.foo.bar.baz;
```

Static access via `::` can be followed by `.` chaining when a static method returns an instance:

```javascript
Factory::create().name;
```

## Lists

### List Literals

```javascript
let empty = [];
let nums = [1, 2, 3];
let mixed = [1, "two", true, null];
```

### List Methods

| Method | Description | Example |
|--------|-------------|---------|
| `.len()` | Get length | `[1, 2, 3].len()` = `3` |
| `.get(index)` | Get element (supports negative indexing) | `[1, 2, 3].get(0)` = `1` |
| `.push(value)` | Return new list with value appended | `[1, 2].push(3)` = `[1, 2, 3]` |
| `.concat(other)` | Return new list with other appended | `[1].concat([2, 3])` = `[1, 2, 3]` |

```javascript
let nums = [10, 20, 30];
nums.len();        // 3
nums.get(0);       // 10
nums.get(-1);      // 30 (negative indexing)
nums.push(40);     // [10, 20, 30, 40]
```

## Strings

### String Methods

| Method | Description | Example |
|--------|-------------|---------|
| `.len()` | Get length | `"hello".len()` = `5` |
| `.concat(other)` | Concatenate strings | `"hi".concat(" there")` = `"hi there"` |

## Built-in Functions

| Function | Description |
|----------|-------------|
| `print(value)` | Print a value to stdout |
| `type(value)` | Get the type as a string |
| `range(end)` | Generate `[0, 1, ..., end-1]` |
| `range(start, end)` | Generate `[start, ..., end-1]` |

## Truthiness

### Falsy Values

- `null`
- `false`
- `0` (integer)
- `0.0` (float)

### Truthy Values

Everything else, including empty strings `""`.
