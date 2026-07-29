# axe VM vs. loxide — Implementation Comparison

A comparison of the **axe** bytecode VM (`src/vm/`) against **loxide**
(`~/working_dir/rust-vs-zig/loxide/`), a near-literal Rust transliteration of
clox (the C VM from *Crafting Interpreters*).

## The headline: two philosophies

- **loxide** is a near-literal port of **clox** into Rust — raw pointers, a
  manual mark-sweep GC, and pointer punning (`#[repr(C)]` struct headers).
- **axe** is idiomatic **safe Rust** — heap objects are handles-as-indices
  (`ObjRef(usize)` into a `Vec<Obj>`), no `unsafe` in the VM, and no GC required
  for memory safety.

That single decision explains almost every difference below.

## Feature / implementation matrix

| Dimension | **axe** (ours) | **loxide** |
|---|---|---|
| Heap objects | `Vec<Obj>`, `ObjRef(usize)` index | `Box`-allocated, `Gc<T>` = `NonNull<T>` raw ptr |
| Memory reclamation | **none — leaks** (heap only grows) | **mark-sweep GC** (`mem.rs`, greystack, `is_marked`) |
| `unsafe` in VM | **zero** | pervasive (raw-ptr stack, ptr punning, manual free) |
| Value stack | `Vec<Value>` (bounds-checked index) | raw `*mut Value` + `top` pointer (no checks) |
| Call frames | `bp: usize` index into stack | `slots_ptr: *mut Value` raw base pointer |
| Closures / upvalues | **none** (`Fn { entry, arity }` is flat) | **full** (`ObjUpvalue`, open/closed, `CloseUpvalue`) |
| Bound methods | none (methods are plain `Fn` + explicit `self`) | `ObjBoundMethod` (first-class `obj.method`) |
| `super` | none | `SuperInvoke` / `GetSuper` |
| Strings | interned at **compile time** → `Symbol(u32)` | interned at **runtime** (`ObjString` + cached FNV hash, pointer-identity equality) |
| Field/method maps | `std::HashMap<Symbol, Value>` | custom open-addressing `Table` keyed by interned-string pointer |
| Code layout | one flat `Bytecode` chunk, funcs inlined w/ `JUMP` | per-function `Chunk` (`ObjFunction.chunk`) |
| Numbers | `Int(i64)` **and** `Float(f64)` | only `f64` (Lox spec) |
| Constant folding | **yes** (our compiler) | no |
| Runtime errors | `panic!` | `Result` + `runtime_error` with line numbers + stack trace |
| Allocator | system | **mimalloc** |

## What loxide does better

### Features / implementation
1. **Real GC** — the biggest gap. axe's `Heap` (`src/vm/vm.rs`) only ever
   `push`es; objects are **never freed**, so any long-running program leaks
   unboundedly. loxide reclaims via mark-sweep, triggered by
   `next_gc` / `bytes_allocated` thresholds.
2. **Closures with upvalues** — axe functions genuinely cannot capture outer
   locals (the `LocalTable` is flat and `Fn` carries no environment). loxide has
   the full clox upvalue machinery. This is a *language* capability we lack, not
   just an optimization.
3. **Bound methods + `super`** — `let m = obj.greet;` works in loxide; methods
   are first-class values. axe can only call them inline.
4. **Runtime error reporting** — loxide keeps `chunk.lines` and prints the source
   line plus a call stack (`runtime_error`). axe `panic!`s with a raw opcode, and
   a type error kills the REPL.

### Performance
5. **Raw-pointer stack & frames** — `push`/`pop`/`peek` are pointer bumps with no
   bounds checks; frame slot access is `slots_ptr.add(i)`. axe pays a bounds
   check on every `stack[bp + slot]`.
6. **Interned-string pointer identity** — property/method/global lookups compare
   a *pointer* (`instance.fields.get(name_ptr)`), with the hash cached on the
   `ObjString`. Cheap and clox-tuned.
7. **mimalloc** — measurably faster allocation than the system allocator under
   object churn.

Net: loxide will out-run axe on allocation-heavy, deeply-nested,
method-dispatch-heavy code — at the cost of being `unsafe` top to bottom.

## What axe does better

1. **Safety** — axe's VM cannot segfault, use-after-free, or corrupt memory.
   loxide can (any bug in the GC or a stale `Gc<T>` is UB). A genuine engineering
   advantage, not a consolation prize.
2. **Compile-time interning is simpler** — resolving names to `Symbol(u32)`
   during parse is cleaner than loxide's runtime string-interning dance, and
   threads no lifetimes.
3. **Constant folding** — axe folds `(10 + 20) * 2 - 5 → 55` at compile time;
   loxide (and clox) don't.
4. **Integer type** — real `i64` arithmetic; Lox is doubles-only.
5. **The index-handle design is a better foundation for a *safe* GC** than
   loxide's raw pointers (see recommendation #1).

## What to adopt, in priority order

### 1. Reclaim memory — but keep it safe (highest impact)
Don't copy loxide's raw-pointer mark-sweep. axe's `ObjRef(usize)` is already a
handle — lean into it. Use a **generational arena** (`slotmap` /
`generational-arena`, or hand-rolled): a free-list reuses slots and a generation
counter makes stale `ObjRef`s detectable instead of UB. Reclamation **and** zero
`unsafe`. The single most valuable thing to take from loxide, done the axe way.

### 2. Line numbers + `Result`-based runtime errors (cheap, big UX win)
Add a parallel `lines: Vec<u32>` to `Bytecode` and convert the VM's `panic!`s
into a `RuntimeError` returned up to the REPL, with the source line — exactly
loxide's `runtime_error`. Low effort; stops type errors from crashing the process.

### 3. Closures / upvalues (biggest *feature* gap, larger effort)
Likely needs per-function chunks (adopt loxide's `ObjFunction { chunk }` split)
plus the upvalue-capture protocol. Sizable refactor, but it's what separates
"toy" from "real" — and upvalues can be implemented with indices rather than
loxide's raw-pointer linked list.

### 4. Faster member lookup (only if profiling says so)
The `HashMap<Symbol, Value>` is fine. If method dispatch shows up hot, consider
caching or a clox-style open-addressing table — a *later* optimization, not a
correctness issue like #1.

### Skip
The raw-pointer stack and manual GC. They buy speed by discarding the safety
that is arguably axe's whole reason to exist in Rust. Get reclamation via
generational indices instead.

## Key file references

**axe** (`src/vm/`): `vm.rs` (Heap/Value/dispatch), `compiler.rs` (constant
folding, OO lowering), `bytecode.rs` (`Constant`), `tables.rs` (compile-time
Global/Local resolution), `instructions.rs`.

**loxide** (`src/`): `mem.rs` (GC + `Gc<T>` + interning), `obj.rs` (object model,
`blacken`/`mark`/`free`), `table.rs` (open-addressing hash table), `value.rs`
(`Value`), `vm.rs` (`CallFrame`, `call`/`invoke`/`capture_upvalue`, `run`).

---

# VM Optimization Catalog (CPython, JVM/HotSpot, and general)

A reference list of optimizations used by mature VMs, grouped by technique. Each
is tagged with axe's status:
- **[have]** — already in axe
- **[easy] / [medium] / [hard]** — worth adopting, with rough effort
- **[N/A]** — only relevant to a JIT or a fundamentally different design

## 1. Value representation

- **Tagged union / enum values** — one word carries type + payload. **[have]**
  (`Value` enum).
- **NaN-boxing** — pack pointers/ints/bools into the unused bits of an IEEE-754
  double, so every `Value` is 8 bytes with no separate tag. Used by LuaJIT,
  SpiderMonkey, JSC; clox has an optional version. **[medium]** (only pays off
  once `Value` is hot and float-heavy).
- **Pointer tagging** — steal low/high bits of aligned pointers to encode small
  ints/tags (V8 SMIs, OCaml). **[medium]**
- **Small-integer caching** — preallocate boxed ints for a common range
  (CPython caches −5..256) so arithmetic reuses shared objects. **[N/A]** for
  axe — our `Int(i64)` is unboxed already, so this is a non-issue (a benefit of
  having a value-type int).
- **Compressed pointers (compressed oops)** — 32-bit object references on a
  64-bit heap (HotSpot). axe's `ObjRef(usize)` could shrink to `u32`. **[easy]**

## 2. Interpreter dispatch (the eval loop)

- **Stack-based bytecode** — compact, simple. **[have]**.
- **Register-based bytecode** — operands name virtual registers, so far fewer
  instructions per expression (Lua 5, Dalvik/ART). Fewer dispatches = faster,
  but a bigger compiler change. **[hard]**
- **Direct/indirect threaded dispatch (computed goto)** — replace the big
  `match` with a jump table of label addresses, eliminating the loop's
  bounds/branch overhead. CPython and HotSpot's template interpreter both do
  this. Rust has no `goto`; approximated with tail calls (`become`, unstable) or
  a `[fn; 256]` dispatch table. **[medium]**
- **Superinstructions** — fuse frequent opcode pairs (e.g. `GET_LOCAL` +
  `GET_PROPERTY`) into one, cutting dispatch count. **[medium]**
- **Token/subroutine threading** — variants of the above. **[medium]**

## 3. Bytecode / compile-time optimizations

- **Constant folding** — evaluate constant expressions at compile time.
  **[have]**.
- **Peephole optimization** — local rewrites over the emitted bytecode (drop
  redundant loads, fold jumps-to-jumps, remove dead `POP`s). CPython runs a
  peephole/optimizer pass. **[easy]** and high-value for axe.
- **Dead code elimination** — drop unreachable code after `return`/`break`.
  **[easy]** (axe currently emits dead `NULL; RETURN` after explicit returns).
- **Constant pool deduplication** — store each literal once. **[have]**
  (`add_constant` dedups).
- **Jump threading / short-circuit compilation** — compile `&&`/`||` and `if`
  with direct conditional jumps instead of materializing bools. **[medium]**
- **Local variable slot allocation** — resolve names to stack slots/indices at
  compile time. **[have]** (`LocalTable`, `GlobalTable`).
- **Marshalled/cached bytecode** — persist compiled bytecode (`.pyc`, class
  files) to skip re-parsing. **[easy]** if you want a `.axc` cache.

## 4. Inline caching & specialization (the biggest interpreter wins)

- **Inline caches (ICs)** — cache the result of a lookup *at the call/access
  site* keyed by observed type, so a repeat hit skips the search. Monomorphic →
  polymorphic → megamorphic. This is *the* central dynamic-language
  optimization (Smalltalk → V8 → CPython 3.11). **[medium]**, very high value.
- **Adaptive specialization / quickening (PEP 659, "Faster CPython")** — the
  interpreter *rewrites its own bytecode* at runtime: `LOAD_ATTR` becomes
  `LOAD_ATTR_INSTANCE_VALUE`, `BINARY_OP` becomes `BINARY_OP_ADD_INT`, once a
  type is observed; it de-specializes if the assumption breaks. Gave CPython
  3.11 ~25% overall. **[hard]** but a great model for where axe could go.
- **Hidden classes / shapes / maps (V8)** — give objects with the same field
  layout a shared "shape" so property access becomes a fixed offset + IC instead
  of a hash lookup. This is how JS makes dynamic objects nearly as fast as
  structs. axe's `HashMap<Symbol,Value>` fields are the natural target. **[hard]**
- **Global/builtin lookup caching** — cache module/global resolution at the site
  (CPython `LOAD_GLOBAL` IC). **[medium]**
- **Polymorphic inline caches for method dispatch** — cache the resolved method
  per receiver type at each call site. **[medium]**

## 5. Calls, methods, and objects

- **Avoid allocating bound-method objects** — CPython's `LOAD_METHOD`/
  `CALL_METHOD` and clox's `OP_INVOKE` fast-path a `obj.m()` call without
  materializing a bound method. axe's `INVOKE` already fuses this. **[have]**.
- **Fast-call / vectorcall protocol (PEP 590)** — pass args on the stack instead
  of building a tuple+dict per call. **[medium]**
- **Zero-cost / lazy frames** — don't heap-allocate a frame object per call;
  build it lazily only if reflected upon (CPython 3.11). axe uses a lightweight
  `Frame` struct already. **[have]** (mostly).
- **Key-sharing / compact instance dicts (PEP 412/3.6)** — instances of one
  class share their key layout, storing only values. **[medium]** (pairs with
  hidden classes).
- **`__slots__`-style fixed layout** — skip the per-instance dict entirely for
  declared fields → array-indexed field access. **[medium]**, big win for a
  statically-shaped object model.
- **Method resolution order (MRO) / type attribute cache** — cache method
  lookups per type, invalidated by a version tag when the class mutates
  (CPython's type cache). **[medium]**

## 6. Strings

- **Interning** — one copy per unique string; compare by identity. **[have]**
  (compile-time `Symbol`).
- **Cached string hashes** — store the hash on the string object so table
  lookups don't rehash (loxide/clox `ObjString.hash`). **[easy]**
- **Compact strings** — Latin-1 vs UTF-16 storage (JVM `CompactStrings`),
  small-string inlining. **[medium]**
- **String deduplication** — GC merges equal strings (HotSpot). **[N/A]** given
  interning already covers identifiers.

## 7. Memory management / GC

- **Bump/arena allocation** — allocate by incrementing a pointer. axe's
  `Vec<Obj>` push is effectively this. **[have]** (but never frees — see the
  comparison above).
- **Thread-local allocation buffers (TLABs)** — per-thread bump regions to avoid
  allocator contention (HotSpot). **[N/A]** (single-threaded).
- **Generational GC** — collect young objects frequently, old rarely (CPython
  cycle collector, all HotSpot collectors). **[hard]**
- **Free lists for hot object types** — recycle frames/tuples/small containers
  (CPython). With axe's index heap, a **free-list of `ObjRef` slots** is the
  natural, safe version. **[medium]** — this is axe's #1 real gap.
- **Reference counting** — immediate reclamation (CPython) + a cycle collector
  for the rest. **[medium]**
- **Concurrent / low-pause collectors** — G1, ZGC, Shenandoah. **[N/A]**.
- **Escape analysis → stack allocation / scalar replacement** — objects that
  don't escape a method are allocated on the stack or dissolved into registers
  (HotSpot). **[N/A]** (JIT-only).

## 8. JIT compilation (beyond interpretation)

- **Tiered compilation** — interpret → cheap baseline JIT (C1) → optimizing JIT
  (C2), promoting hot code by profiling counters. **[hard]**
- **Profile-guided optimization** — invocation/backedge counters, per-site type
  profiles drive inlining and speculation. **[hard]**
- **Aggressive method inlining** — the single most important JIT optimization;
  exposes everything else. **[hard]**
- **On-stack replacement (OSR)** — swap a running hot loop from interpreter to
  compiled code mid-execution. **[hard]**
- **Deoptimization** — bail back to the interpreter when a speculative
  assumption (monomorphic type, no override) is violated. **[hard]**
- **Devirtualization via CHA** — turn virtual calls into direct/inlined ones when
  class-hierarchy analysis proves a single target. **[hard]**
- **Classic scalar opts in the JIT** — GVN/CSE, LICM, loop unrolling,
  range-check elimination, dead-code elimination, register allocation, null-check
  elimination, autovectorization. **[hard]**
- **Intrinsics** — hand-written machine code for hot library methods (`Math.*`,
  `System.arraycopy`, `String` ops). **[hard]**
- **Tracing JIT** — compile hot *paths* rather than methods (PyPy, LuaJIT).
  **[hard]**
- **Copy-and-patch JIT** — stitch precompiled machine-code stencils for very fast
  baseline compilation (CPython 3.13 experimental JIT). **[hard]**

## Recommended path for axe (interpreter, no JIT)

The high-leverage, safety-preserving wins, in order:

1. **Free-list reclamation** over the `ObjRef` index heap (§7) — fixes the leak,
   stays `unsafe`-free.
2. **Peephole + dead-code elimination** (§3) — cheap, immediate.
3. **Computed-goto-style dispatch table** (§2) — removes match/loop overhead.
4. **Cached string hashes** (§6) — trivial, speeds every member lookup.
5. **Inline caches for property/method access** (§4) — the biggest interpreter
   speedup before you'd ever need a JIT.
6. (Long game) **hidden classes / fixed instance layout** (§4/§5) to turn
   `obj.field` from a hash lookup into an indexed load.

Everything JIT-related (§8) is out of scope until axe is a mature interpreter;
the §4 techniques are how you get most of the speed without one.
