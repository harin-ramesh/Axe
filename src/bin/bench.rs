//! Micro-benchmark harness comparing the stack VM against the tree-walker.
//!
//! Run with:  cargo run --release --bin bench
//!
//! Each benchmark is a small Axe program run end-to-end (parse is done once;
//! each timed iteration re-compiles + executes for the VM, and re-resolves +
//! evaluates for the tree-walker — mirroring how each backend actually works).
//! Iterations auto-scale to a fixed time budget so both fast and slow programs
//! get a stable per-iteration figure.

use std::time::{Duration, Instant};

use axe::{Axe, AxeVM, Compiler, Context, Parser};

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
                      fn bump(self) { self.count = self.count + 1; self.count; }
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
    println!(
        "{:<42} {:>14} {:>14} {:>10}",
        "benchmark", "VM (µs/iter)", "TW (µs/iter)", "speedup"
    );
    println!("{}", "-".repeat(82));

    for bench in benchmarks() {
        // One shared context so parsed Symbols are valid for both backends.
        let ctx = Context::new();
        let program = match Parser::new(bench.src, &ctx).parse() {
            Ok(p) => p,
            Err(e) => {
                println!("{:<42} parse error: {}", bench.name, e);
                continue;
            }
        };

        // VM: compile + execute each iteration.
        let (vm_iters, vm_time) = measure(|| {
            let bytecode = Compiler::new(&ctx).compile(&program);
            let mut vm = AxeVM::new(&bytecode);
            let _ = vm.exec();
        });
        let vm_us = per_iter_us(vm_iters, vm_time);

        // Only benchmark the tree-walker if it can actually run the program.
        // (It can't run OO — instance methods return null there — so timing it
        // would just measure how fast it errors out.)
        let tw_ok = Axe::new(&ctx).run(program.clone()).is_ok();
        if tw_ok {
            let (tw_iters, tw_time) = measure(|| {
                let mut tw = Axe::new(&ctx);
                let _ = tw.run(program.clone());
            });
            let tw_us = per_iter_us(tw_iters, tw_time);
            let speedup = tw_us / vm_us;
            println!(
                "{:<42} {:>14.2} {:>14.2} {:>9.2}x",
                bench.name, vm_us, tw_us, speedup
            );
        } else {
            println!(
                "{:<42} {:>14.2} {:>14} {:>10}",
                bench.name, vm_us, "n/a", "VM-only"
            );
        }
    }
}
