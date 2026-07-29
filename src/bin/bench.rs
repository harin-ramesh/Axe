//! Micro-benchmark harness for the stack VM.
//!
//! Run with:  cargo run --release --bin bench
//!
//! Each benchmark is a small Axe program run end-to-end (parse is done once;
//! each timed iteration re-compiles + executes). Iterations auto-scale to a
//! fixed time budget so both fast and slow programs get a stable
//! per-iteration figure.

use std::time::{Duration, Instant};

use axe::{AxeVM, Compiler, Context, Parser};

/// Wall-clock budget each backend gets per benchmark before we stop iterating.
const BUDGET: Duration = Duration::from_millis(600);

struct Benchmark {
    name: &'static str,
    src: &'static str,
}

fn benchmarks() -> Vec<Benchmark> {
    vec![
        Benchmark {
            name: "fib(28) [recursion/calls]",
            src: "fn fib(n) { if (n < 2) { return n; } return fib(n - 1) + fib(n - 2); }
                  fib(28);",
        },
        Benchmark {
            name: "while sum to 1_000_000 [tight loop]",
            src: "let i = 0; let sum = 0;
                  while (i < 1000000) { sum = sum + i; i = i + 1; }
                  sum;",
        },
        Benchmark {
            name: "nested loops 1000x1000 [loop dispatch]",
            src: "let acc = 0;
                  let a = 0;
                  while (a < 1000) {
                      let b = 0;
                      while (b < 1000) { acc = acc + 1; b = b + 1; }
                      a = a + 1;
                  }
                  acc;",
        },
        Benchmark {
            name: "for range(0, 200000) sum [loop + alloc]",
            src: "let total = 0;
                  for n in range(0, 200000) { total = total + n; }
                  total;",
        },
        Benchmark {
            name: "OO: 100k instances + method calls",
            src: "class Counter {
                      fn init(self, start) { self.count = start; }
                      fn bump(self) { self.count = self.count + 1; return self.count; }
                  }
                  let i = 0; let acc = 0;
                  while (i < 100000) {
                      let c = new Counter(i);
                      acc = acc + c.bump();
                      i = i + 1;
                  }
                  acc;",
        },
    ]
}

/// Run `f` repeatedly until BUDGET elapses; return (iterations, elapsed).
fn measure(mut f: impl FnMut()) -> (u64, Duration) {
    f(); // warm up (caches, first-touch allocation)
    let start = Instant::now();
    let mut iters = 0u64;
    while start.elapsed() < BUDGET {
        f();
        iters += 1;
    }
    (iters.max(1), start.elapsed())
}

fn per_iter_us(iters: u64, elapsed: Duration) -> f64 {
    elapsed.as_secs_f64() * 1e6 / iters as f64
}

fn main() {
    println!("{:<42} {:>14}", "benchmark", "VM (µs/iter)");
    println!("{}", "-".repeat(57));

    for bench in benchmarks() {
        let ctx = Context::new();
        let program = match Parser::new(bench.src, &ctx).parse() {
            Ok(p) => p,
            Err(e) => {
                println!("{:<42} parse error: {}", bench.name, e);
                continue;
            }
        };

        // Compile + execute each iteration.
        let (vm_iters, vm_time) = measure(|| {
            let bytecode = Compiler::new(&ctx)
                .compile(&program)
                .expect("compile error");
            let mut vm = AxeVM::new(&bytecode);
            vm.exec().expect("runtime error");
        });
        let vm_us = per_iter_us(vm_iters, vm_time);
        println!("{:<42} {:>14.2}", bench.name, vm_us);
    }
}
