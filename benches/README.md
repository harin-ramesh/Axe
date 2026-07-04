# Benchmarks

## `vm_bench` — regression tracking (criterion)

Tracks VM performance over development. Criterion saves results under
`target/criterion/` and, on each run, reports the % change vs. the previous run
(or a named baseline) and flags regressions/improvements.

```sh
# Run all benchmarks (compares against the last run automatically)
cargo bench

# Before a change you want to measure: save a named baseline
cargo bench -- --save-baseline before

# ...make your change, then compare against it
cargo bench -- --baseline before

# Run a subset (regex filter over "group/name")
cargo bench --bench vm_bench -- vm_exec
cargo bench --bench vm_bench -- fib_25
```

Groups:
- **`vm_exec`** — VM execution only (bytecode compiled once). Purest signal for
  runtime regressions.
- **`vm_end_to_end`** — compile + execute (what a user pays).
- **`tree_walker`** — the tree-walker, for the VM-vs-TW comparison. Skips
  programs it can't run (e.g. OO).

Typical regression workflow:
```sh
git checkout main && cargo bench -- --save-baseline main
git checkout my-branch && cargo bench -- --baseline main   # shows deltas
```

## `bench` — quick VM-vs-TW table (custom, no history)

A human-readable one-shot comparison printed as a table. No baselines/history —
use `cargo bench` for regression tracking.

```sh
cargo run --release --bin bench
```
