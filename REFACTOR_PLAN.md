# AST Refactor Plan

## New Types in `src/eval.rs`

```rust
// Top-level program
pub struct Program {
    pub stmts: Vec<Stmt>,
}

// Literals (primitive values)
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

// Unary operations
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Neg,  // -x
}

// Expressions (things that produce values)
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    List(Vec<Expr>),
    Var(String),
    Binary(Operation, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Lambda(Vec<String>, Vec<Stmt>),
    Property(Box<Expr>, String),
    New(String, Vec<Expr>),
}

// Statements
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr),
    Block(Vec<Stmt>),
    Let(String, Expr),
    Assign(String, Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    Function(String, Vec<String>, Vec<Stmt>),
    Class(String, Option<String>, Vec<Stmt>),
}

// Runtime values
pub enum Value {
    Literal(Literal),
    List(Vec<Value>),
    Function(Vec<String>, Vec<Stmt>, EnvRef),
    NativeFunction(String, fn(&[Value]) -> Result<Value, &'static str>),
    Object(EnvRef),  // Renamed from Environment - used for class instances
}
```

## Syntactic Sugar (handled in parser)

| Sugar | Desugars to |
|-------|-------------|
| `i++` | `Stmt::Assign("i", Expr::Binary(Add, Var("i"), Literal(Int(1))))` |
| `i--` | `Stmt::Assign("i", Expr::Binary(Sub, Var("i"), Literal(Int(1))))` |
| `for (init; cond; update) { body }` | `Stmt::Block([init, While(cond, [body..., update])])` |

## Files to Change

### 1. `src/eval.rs`
- Replace current `Expr`, `Condition`, `Value` enums with new structure
- Add `Program`, `Literal`, `UnaryOp`, `Stmt` types
- Update `Axe::eval()` to take `Program` instead of `Expr`
- Add `eval_stmt()` and `eval_expr()` methods
- Update all `impl` blocks for `Value` (Clone, Debug, PartialEq, Display)
- Remove `eval_condition()` (use `eval_expr()` instead)
- Rename `Value::Environment` to `Value::Object`

### 2. `src/parser.rs`
- Update `parse()` to return `Program`
- Rename/update methods to return `Stmt` or `Expr` appropriately
- Handle `Inc`/`Dec`/`For` desugaring in parser
- Remove `Condition` parsing, use `Expr` for conditions

### 3. `src/transformer.rs`
- Update to work with new `Stmt`/`Expr` types
- May become simpler or unnecessary if sugar is handled in parser

### 4. `src/lib.rs`
- Update exports: `Program`, `Stmt`, `Expr`, `Literal`, `UnaryOp`, etc.

### 5. `tests/*.rs` (all test files)
- Update assertions to use new structure
- Example change:
  - Before: `Expr::Block(vec![Expr::Int(42)])`
  - After: `Program { stmts: vec![Stmt::Expr(Expr::Literal(Literal::Int(42)))] }`

## Key Decisions Made

1. **`Inc`/`Dec`** - Syntactic sugar, desugar in parser
2. **`For`** - Syntactic sugar, desugar to `Block + While` in parser
3. **`Value::Environment`** - Rename to `Value::Object` for clarity
4. **`Condition` enum** - Remove entirely, use `Expr` for conditions in `If`/`While`
5. **Unary minus** - Use `Unary(Neg, x)` for cleaner representation

## Notes

- `Value::Object(EnvRef)` is used for class definitions and instances
- The `EnvRef` inside objects stores fields/methods as key-value pairs
- Native functions remain unchanged in signature
