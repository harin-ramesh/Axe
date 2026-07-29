# Lists in the axe VM: representation & performance

Notes on how the VM represents lists, why `Vec<Value>` is the right backing
store, and the concrete levers for making list-heavy code faster.

## Representation: `Obj::List(Vec<Value>)`

A list is a **managed heap object** (`Obj::List`, referenced by an `ObjRef`
handle) whose elements live in a **`Vec<Value>` backing buffer**. Two layers:

- `Obj::List` = the list *object* (identity, GC-tracked, shared by handle).
- `Vec<Value>` = the raw growable element buffer.

This is the same two-layer shape every production runtime uses — the object and
its backing array are separate allocations:

| Runtime | List object | Backing array |
|---|---|---|
| CPython | `PyListObject` (refcount, type, `ob_size`) | `malloc`'d `PyObject **ob_item` + `allocated` |
| JVM `ArrayList` | the `ArrayList` object | `Object[] elementData` + `size` |
| V8 arrays | `JSArray` | a C++ `FixedArray` backing store |
| Lua tables | `Table` | a C array part (+ hash part) |
| **axe** | `Obj::List` | `Vec<Value>` |

### "A Python list is not a C array" — correct, and axe matches it

A Python `list` is not itself a C array; it is a heap object that *points to* a
separately-allocated C array of `PyObject*`. `[1, 2, 3]` is three levels:
1. the `PyListObject` (the list),
2. the `ob_item` C array of pointers (the buffer),
3. the boxed element objects the pointers point to.

axe mirrors levels 1–2 exactly (`Obj::List` + the `Vec`'s heap buffer) and is
**better on level 3**: `Value::Int(i64)` is stored *inline* in the `Vec`, so
there is no third allocation and no pointer chase. A list-of-ints in axe is one
indirection shallower than in CPython (which caches small ints −5..256 but still
stores pointers).

## Is `Vec` the right backing? Yes.

A list's contract is a *growable, contiguous, indexable* sequence. `Vec` is
exactly that, with an optimal amortized-O(1) growth strategy. The alternatives
are worse for axe's semantics:

- `Box<[Value]>` — can't grow; a list must.
- linked list (the Lisp/C sense of "list") — cache-hostile, one allocation per
  element. Strictly worse.
- `Rc<RefCell<Vec>>` — redundant: the arena (`Vec<Obj>` + `ObjRef`) already
  gives shared, mutable-by-handle access. Reference counting you don't need.
- hand-rolled growable array — reimplementing `Vec` with no upside.

`Vec` would only be wrong for semantics axe doesn't have:
- **lazy sequences** (Python's `range`) → not a list; use a small iterator object.
- **persistent/immutable lists** with structural sharing (Clojure/Haskell) → an
  RRB-tree / persistent vector.
- **O(1) push/pop at both ends** (deque) → `VecDeque`.

**One honest caveat** (about the *arena*, not `Vec`): mutating a list is
`heap.get_mut(ref)` → `&mut Vec`, and you can't hold that `&mut` while also
allocating another heap object in the same expression. Fine for `push`/`get`/
`set`; only bites operations that allocate *while* mutating.

## The real smell (if any): lists are Rust-only citizens

The `Vec` isn't the problem. The asymmetry is that list logic currently lives in
Rust, not in axe: `range` builds the list with `(start..end).collect()`, and
there's **no way to manipulate a list from axe** (no `push`/`get`/`set`, no
indexing syntax). Fix = give lists a first-class language API; the `Vec` stays a
hidden implementation detail. That's a language-surface change, not a storage
change.

## Performance levers for list-heavy code (ranked)

### 1. Shrink `Value` (biggest structural win, helps everything)
`Value` is ~24–32 bytes, inflated by the `&'static str` name inside
`Native(&'static str, NativeFn)` (a 16-byte fat pointer carried only for
`display`). Since both the operand stack and lists are `Vec<Value>`, every copy /
`GET_INDEX` / push / pop moves that many bytes. Drop the name from `Native`
(store nothing, or a small interned id / `u8` index into the builtins table) and
the enum shrinks to ~16 bytes — roughly **halving memory traffic and cache
footprint** VM-wide. Small change, highest payoff/effort ratio.

### 2. Hoist `LEN` out of the `for` loop (easy, targeted)
The `for` desugar currently recomputes `len(list)` **every iteration** (a heap
deref each pass). Compute it once into a hidden local before the loop. This is
why `for range` benched at only ~1.6× over the tree-walker while `while` hit
~5×. Small compiler change; immediate win for every range/list loop.

### 3. Lazy `range` (kills the O(n) allocation)
The dominant cost of `for i in range(0, N)` is allocating an N-element `Vec`. A
lazy range object (holds `start/stop/step`, yields via a `FOR_ITER`-style
opcode — the CPython approach) makes the loop allocate **O(1)** and skips the
list entirely: no `Vec`, no `GET_INDEX`, no bounds check, no clone. Largest
speedup available for range-driven loops. More effort (new object + iterator
opcodes) but it's the correct fix.

### 4. NaN-boxing `Value` → 8 bytes (the ceiling, high effort)
Pack every `Value` into one 64-bit word (ints/bools/handles in the unused bits
of an f64 NaN). ~24 → 8 bytes = ~3× less stack/list memory traffic; the biggest
possible win for memory-bound list code. `unsafe` and fiddly — do it only after
profiling says memory bandwidth is the bottleneck. (LuaJIT / SpiderMonkey / JSC
do this.)

### Not worth it
- Swapping `Vec` for another container — it's already optimal.
- `get_unchecked` to skip bounds checks — trades the safety story for a check
  that's nearly free once predicted.
- Reserving capacity in `range`/`BUILD_LIST` — already exact
  (`(start..end).collect()` reserves via `ExactSizeIterator`).

## Recommended order
Do **#1 (shrink `Value`)** and **#2 (hoist `LEN`)** now — small, pure wins, no
risk. Then **#3 (lazy range)** if range-loops are a real workload. Save
**#4 (NaN-boxing)** for when the benchmark says bytes are the bottleneck.
