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
cargo run --release <filename>.ax

# Using compiled binary
./target/release/axe <filename>.ax
```

## Your First Program

Create `hello.ax`:

```javascript
// My first Axe program
let message = "Hello, World!";
let x = 10;
let y = 20;
let sum = x + y;
```

Run it:

```bash
cargo run --release hello.ax
```

## Example Files

The `examples/` directory contains sample programs:

| File | Description |
|------|-------------|
| `hello.ax` | Hello world |
| `fibonacci.ax` | Arithmetic sequences |
| `simple_counter.ax` | Variable assignment |
| `builtins.ax` | Operators demo |
| `scoping_explained.ax` | Block scoping guide |

```bash
cargo run --release examples/fibonacci.ax
```

## Next Steps

- [Language Reference](language-reference.md) - Full syntax guide
- [Examples](examples.md) - More code samples
