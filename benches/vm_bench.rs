//! Criterion benchmarks for tracking VM performance across development.
//!
//! Run:            cargo bench
//! Save baseline:  cargo bench -- --save-baseline main
//! Compare to it:  cargo bench -- --baseline main
//!
//! Criterion stores results in `target/criterion/` and, on each run, reports the
//! percentage change versus the previous run (or a named baseline), flagging
//! regressions/improvements — which is what makes this useful for catching
//! performance regressions over time.
//!
//! Three groups:
//!   - `vm_exec`        : VM execution only (bytecode compiled once) — the purest
//!                        signal for runtime regressions.
//!   - `vm_end_to_end`  : compile + execute — what a user actually pays.
//!   - `tree_walker`    : the tree-walker, for the VM-vs-TW comparison (skips
//!                        programs it can't run, e.g. OO).

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use axe::{Axe, AxeVM, Compiler, Context, Parser};

/// (name, source). Sized so each runs in a few ms — small enough for criterion's
/// sampling to stay fast, large enough to dominate fixed overhead.
const WORKLOADS: &[(&str, &str)] = &[
    (
        "fib_25",
        "fn fib(n) { if (n < 2) { return n; } return fib(n - 1) + fib(n - 2); }
         fib(25);",
    ),
    (
        "while_sum_100k",
        "let i = 0; let sum = 0;
         while (i < 100000) { sum = sum + i; i = i + 1; }
         sum;",
    ),
    (
        "nested_loops_300x300",
        "let acc = 0; let a = 0;
         while (a < 300) {
             let b = 0;
             while (b < 300) { acc = acc + 1; b = b + 1; }
             a = a + 1;
         }
         acc;",
    ),
    (
        "for_range_50k",
        "let total = 0;
         for n in range(0, 50000) { total = total + n; }
         total;",
    ),
    (
        "oo_20k_instances",
        "class Counter {
             fn init(self, start) { self.count = start; }
             fn bump(self) { self.count = self.count + 1; self.count; }
         }
         let i = 0; let acc = 0;
         while (i < 20000) {
             let c = new Counter(i);
             acc = acc + c.bump();
             i = i + 1;
         }
         acc;",
    ),
];

/// VM execution only: bytecode is compiled once, outside the timing loop.
fn bench_vm_exec(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_exec");
    group.sample_size(50);
    for (name, src) in WORKLOADS {
        let ctx = Context::new();
        let program = Parser::new(src, &ctx).parse().expect("parse");
        let bytecode = Compiler::new(&ctx).compile(&program);
        group.bench_function(*name, |b| {
            b.iter(|| {
                let mut vm = AxeVM::new(black_box(&bytecode));
                black_box(vm.exec())
            });
        });
    }
    group.finish();
}

/// Compile + execute, mirroring the real user-facing cost.
fn bench_vm_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_end_to_end");
    group.sample_size(50);
    for (name, src) in WORKLOADS {
        let ctx = Context::new();
        let program = Parser::new(src, &ctx).parse().expect("parse");
        group.bench_function(*name, |b| {
            b.iter(|| {
                let bytecode = Compiler::new(&ctx).compile(black_box(&program));
                let mut vm = AxeVM::new(&bytecode);
                black_box(vm.exec())
            });
        });
    }
    group.finish();
}

/// Tree-walker, for the VM-vs-TW comparison. Skips programs it can't run.
fn bench_tree_walker(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_walker");
    group.sample_size(50);
    for (name, src) in WORKLOADS {
        let ctx = Context::new();
        let program = Parser::new(src, &ctx).parse().expect("parse");
        // Skip anything the tree-walker can't actually run (e.g. OO methods).
        if Axe::new(&ctx).run(program.clone()).is_err() {
            continue;
        }
        group.bench_function(*name, |b| {
            b.iter(|| {
                let mut tw = Axe::new(&ctx);
                black_box(tw.run(program.clone()))
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_vm_exec,
    bench_vm_end_to_end,
    bench_tree_walker
);
criterion_main!(benches);
