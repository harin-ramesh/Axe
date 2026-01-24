# Getting Started

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

## Installation

```bash
git clone <repository-url>
cd axe
cargo build --release
```

The binary will be at `./target/release/axe`.

## Running Programs

```bash
# Using cargo
cargo run --release <filename>.axe

# Using compiled binary
./target/release/axe <filename>.axe
```

## Your First Program

Create `hello.axe`:

```javascript
// My first Axe program
let message = "Hello, World!";
let x = 10;
let y = 20;
let sum = x + y;
```

Run it:

```bash
cargo run --release hello.axe
```

## Example Files

The `examples/` directory contains sample programs:

| File | Description |
|------|-------------|
| `hello.axe` | Hello world |
| `fibonacci.axe` | Arithmetic sequences |
| `simple_counter.axe` | Variable assignment |
| `builtins.axe` | Operators demo |
| `scoping_explained.axe` | Block scoping guide |

```bash
cargo run --release examples/fibonacci.axe
```

## Next Steps

- [Language Reference](language-reference.md) - Full syntax guide
- [Examples](examples.md) - More code samples
