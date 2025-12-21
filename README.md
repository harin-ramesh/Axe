# Eva

A lightweight interpreter written in Rust featuring scoped variables, arithmetic operations, and block expressions.

## Features

- **Arithmetic Operations**: Add, subtract, multiply, and divide integers and floats
- **Variables**: Define and reference variables with lexical scoping
- **Block Expressions**: Create nested scopes with block expressions
- **Variable Shadowing**: Inner scopes can shadow outer scope variables
- **Type Safety**: Strong type checking with compile-time guarantees

## Installation

```bash
cargo build --release
```

## Usage

### As a Library

```rust
use eva::{Eva, Expr, Value, Operation};

fn main() {
    let eva = Eva::new();
    
    // Simple arithmetic
    let expr = Expr::Binary(
        Operation::Add,
        Box::new(Expr::Int(10)),
        Box::new(Expr::Int(20))
    );
    let result = eva.eval(expr).unwrap();
    assert_eq!(result, Value::Int(30));
    
    // Variables
    eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(42)))).unwrap();
    let result = eva.eval(Expr::Var("x".into())).unwrap();
    assert_eq!(result, Value::Int(42));
    
    // Block expressions with scoping
    let block = Expr::Block(vec![
        Expr::Set("y".into(), Box::new(Expr::Int(100))),
        Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Var("y".into())),
            Box::new(Expr::Int(2))
        )
    ]);
    let result = eva.eval(block).unwrap();
    assert_eq!(result, Value::Int(200));
}
```

## Expression Types

### Values

- `Expr::Null` - Null value
- `Expr::Int(i64)` - Integer literal
- `Expr::Float(f64)` - Float literal
- `Expr::Str(String)` - String literal

### Operations

- `Expr::Binary(op, left, right)` - Binary operations (Add, Sub, Mul, Div)
- `Expr::Set(name, expr)` - Variable assignment
- `Expr::Var(name)` - Variable reference
- `Expr::Block(exprs)` - Block expression creating a new scope

## Scoping Rules

Eva implements lexical scoping with proper variable shadowing:

### Parent Scope Access
```rust
// Child blocks can access parent variables
let eva = Eva::new();

let parent_block = Expr::Block(vec![
    Expr::Set("x".into(), Box::new(Expr::Int(10))),
    Expr::Block(vec![
        Expr::Var("x".into()) // Returns 10
    ])
]);
```

### Variable Shadowing
```rust
// Inner scopes can shadow outer variables without modifying them
let eva = Eva::new();
eva.eval(Expr::Set("x".into(), Box::new(Expr::Int(1)))).unwrap();

let block = Expr::Block(vec![
    Expr::Set("x".into(), Box::new(Expr::Int(100))), // Shadows global x
    Expr::Var("x".into()) // Returns 100
]);

eva.eval(block).unwrap(); // Returns Value::Int(100)
let global_x = eva.eval(Expr::Var("x".into())).unwrap();
// global_x is still 1 - unchanged by the block
```

### Scope Isolation
```rust
// Variables defined in blocks don't leak to parent scope
let eva = Eva::new();

let block = Expr::Block(vec![
    Expr::Set("local_var".into(), Box::new(Expr::Int(42)))
]);

eva.eval(block).unwrap();
// local_var is not accessible here
eva.eval(Expr::Var("local_var".into())).unwrap_err();
```

## Implementation Details

### Environment Model

Eva uses a parent-pointer tree structure for environments:

```rust
type EnvRef = Rc<RefCell<Environment>>;

pub struct Environment {
    records: HashMap<String, Value>,
    parent: Option<EnvRef>,
}
```

The `Rc<RefCell<T>>` pattern enables:
- **Shared Ownership**: Multiple references to the same environment
- **Interior Mutability**: Mutation through shared references
- **Variable Persistence**: Changes persist across expressions in the same scope

When cloning `EnvRef`, only the reference count is incremented - the underlying `Environment` is shared. This allows variables to persist within a block while maintaining proper scoping.

### Variable Name Validation

Variable names must follow these rules:
- Start with a letter (a-z, A-Z) or underscore (_)
- Contain only letters, digits (0-9), or underscores
- Match regex: `^[a-zA-Z_][a-zA-Z0-9_]*$`

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test block_tests
cargo test --test variable_tests
cargo test --test eval_tests
```

### Test Coverage

- **Block Tests**: Scoping, nesting, shadowing, variable persistence
- **Variable Tests**: Assignment, retrieval, scope isolation
- **Evaluation Tests**: Arithmetic operations, type checking, error handling
- **Validation Tests**: Variable name validation rules

## Error Handling

Eva returns `Result<Value, &'static str>` with descriptive error messages:

- `"undefined variable"` - Variable not found in current or parent scopes
- `"invalid variable name"` - Variable name doesn't match validation rules
- `"division by zero"` - Division or modulo by zero
- `"type error"` - Type mismatch in operations

## Architecture

```
eva/
├── src/
│   ├── lib.rs       # Core interpreter implementation
│   └── main.rs      # CLI entry point
├── tests/
│   ├── block_tests.rs              # Block expression tests
│   ├── eval_tests.rs               # Evaluation tests
│   ├── variable_tests.rs           # Variable tests
│   └── variable_validation_tests.rs # Name validation tests
├── Cargo.toml
└── README.md
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
