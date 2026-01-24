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

```javascript
true && false   // false
true || false   // true
```

### Bitwise

| Operator | Description |
|----------|-------------|
| `&` | Bitwise AND |
| `\|` | Bitwise OR |

```javascript
5 & 3   // 1
5 | 3   // 7
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

## Truthiness

### Falsy Values

- `null`
- `false`
- `0` (integer)
- `0.0` (float)

### Truthy Values

Everything else, including empty strings `""`.
