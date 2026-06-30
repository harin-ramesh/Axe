# Adding a Managed Object Heap to the Axe VM

A guide for giving the bytecode VM its own **heap** — a place to allocate
dynamically-sized, long-lived values (strings, lists, objects, closures) that
outlive the operand stack. This replaces the current `Rc<Obj>` approach with a
VM-owned heap addressed by **handles** (indices).

This is the standard design used by clox, CPython, and the JVM, and it's the
prerequisite for lists, instances, closures, and garbage collection.

---

## 1. Why bother (the payoff)

You already have a heap — it's just `Rc`-shaped (`Value::Obj(Rc<Obj>)`). Moving
to a VM-owned heap buys you:

1. **Unblocks the language.** Lists, objects/instances, and closures are all
   heap-allocated variable-sized things. The tree-walker already has
   `List`/`Object`/`Function` values; the VM can't represent them until it has a
   heap to put them in.
2. **Enables garbage collection.** `Rc` cannot collect reference **cycles**
   (A → B → A leaks forever). A VM-owned heap lets you write a tracing GC that
   walks roots and frees the unreachable — impossible when the runtime doesn't
   own the objects.
3. **No refcount traffic.** Every `Value` clone today bumps/drops a counter.
   A `Copy` handle (`u32`) is a plain integer copy — no bookkeeping.
4. **Cache locality (the DoD win).** A 4-byte handle beats an 8-byte pointer,
   and objects sit contiguously in one `Vec` instead of scattered across
   separate allocations. Denser stack + denser heap = fewer cache misses.
5. **Cheap identity.** "Same object?" is an integer compare on the handle.
6. **You control the memory model** — allocation, fragmentation, when GC runs,
   object layout.

**Cost to be aware of:** handles can dangle if you free/compact carelessly.
Mitigations covered in §6.

---

## 2. The core idea

```
Value::Obj(Rc<Obj>)          ──►   Value::Obj(ObjRef)
                                   where ObjRef(u32) is an index

(objects scattered in Rc allocs)   heap: Vec<Obj>  owned by the VM
```

- A **handle** (`ObjRef`) is just an index into the VM's `heap: Vec<Obj>`.
- To **allocate**: push onto `heap`, return the index.
- To **read**: index into `heap` with the handle.
- The VM owns every object's lifetime — nothing is freed until GC says so.

---

## 3. Implementation phases

Do these in order; each phase compiles and runs on its own.

### Phase A — Introduce the handle and heap, migrate strings

This is purely a refactor: same behavior, different representation. Strings are
the only heap object today, so the blast radius is small.

1. Define `ObjRef(u32)` — derive `Clone, Copy, PartialEq, Eq, Debug`.
2. Change `Value::Obj(Rc<Obj>)` to `Value::Obj(ObjRef)`.
3. Add `heap: Vec<Obj>` to the `AxeVM` struct; initialize empty in `new`.
4. Add two helpers:
   - `alloc(&mut self, obj: Obj) -> ObjRef` — push, return index.
   - `deref(&self, r: ObjRef) -> &Obj` — index into the heap.
5. Fix every site that constructed or read an `Obj`:
   - **String concat in `ADD`**: allocate the new `Obj::Str` and push
     `Value::Obj(handle)`.
   - **`as_bool` / `display` / `PartialEq`**: these need the heap to look
     through a handle (see §5 — they currently take `&self` with no heap).
   - **`read_constant`**: see §4 — string constants need a home in the heap.

**Checkpoint:** all existing tests pass with no `Rc` in sight.

### Phase B — Add list values

1. Add `Obj::List(Vec<Value>)`.
2. Compile list literals (`ExprKind::List`) to: evaluate each element (pushing
   onto the stack), then a new `MAKE_LIST n` opcode.
3. `MAKE_LIST n`: pop `n` values, `alloc(Obj::List(values))`, push the handle.
4. Add indexing opcodes (`INDEX_GET`, `INDEX_SET`) as needed.

### Phase C — Objects / instances and closures

Same pattern: add `Obj::Instance { .. }`, `Obj::Closure { .. }`, opcodes to
build them, and field/upvalue access that derefs the handle.

### Phase D — Garbage collection (see §6)

---

## 4. Where string constants live

Today, string constants sit in `Bytecode.constants` as `Value::Obj(Rc<Obj>)`.
With a heap, a `Value::Obj(handle)` is meaningless until the heap actually
contains that object. Two options:

- **Option 1 (simplest): intern constants into the heap at VM startup.**
  Keep string *literals* in the constant pool as raw `String`s (a separate
  `Vec<String>` or an `Obj`-free constant kind). When `CONST` runs for a string,
  `alloc` it into the heap and push the handle. Downside: re-allocates on every
  execution of that `CONST`.
- **Option 2: preload constants into the heap once, store handles in the pool.**
  Before running, walk the constant pool, move each string `Obj` into the heap,
  and rewrite the constant to `Value::Obj(handle)`. `CONST` then just clones a
  handle. Cleaner and faster; do this once the basics work.

Start with Option 1, switch to Option 2 later.

---

## 5. The `&self` → `&heap` ripple

Methods like `as_bool`, `display`, and `PartialEq` currently inspect an `Obj`
directly through the `Rc`. After the change, a `Value::Obj` is just a number —
those methods need access to the heap to interpret it.

Plan for this:

- Move object-aware logic into methods **on the VM** (which owns the heap), e.g.
  `vm.is_truthy(value)`, `vm.values_eq(a, b)`, `vm.display(value)`, rather than
  on `Value` itself.
- Or pass `&heap` into those functions.
- `PartialEq for Value` can no longer compare object *contents* without the heap.
  Decide your semantics:
  - **Identity equality** (`a.0 == b.0`) needs no heap — two handles are equal
    iff they point to the same object. Simple, but `"ab" == "ab"` (two separate
    allocations) would be `false`.
  - **Structural equality** (compare string bytes / list elements) needs the
    heap — implement it as a VM method, not via `derive(PartialEq)`.

Pick identity for objects generally, structural for strings if your language
expects value-equality on strings (most do). Interning strings (Option 2 above)
lets identity equality stand in for string value equality.

---

## 6. Garbage collection (mark-and-sweep)

Once mutable objects/closures exist, you can create cycles, so refcounting isn't
enough. A basic mark-sweep:

### Roots — everything reachable "from outside" the heap
- the operand `stack`
- the `globals` array
- values held in call `frames` (if any)
- (later) any temporaries the VM is mid-operation on

### Mark phase
1. Give each `Obj` a `marked: bool` (or keep a side `Vec<bool>` parallel to the
   heap — the DoD-friendlier layout).
2. Start from the roots; for every `Value::Obj(handle)`, mark it, then
   **recursively mark** objects it references (list elements, instance fields,
   closure upvalues). Use an explicit worklist (`Vec<ObjRef>`) instead of
   recursion to avoid stack overflow on deep structures.

### Sweep phase
- Walk the heap. Every unmarked object is unreachable → free it.
- Clear marks for the next cycle.

### Handle stability — the key decision
Sweeping creates **holes**. You must not let a freed slot's index later point to
a different object, or old handles silently corrupt. Two approaches:

- **Free-list (recommended first):** don't move objects. Replace freed slots
  with a `Obj::Free(next_free_index)` tombstone and keep a free-list head.
  `alloc` reuses a free slot if available, else pushes. Indices never change, so
  all existing handles stay valid.
- **Generational handles:** make `ObjRef { index, generation }`. Bump a slot's
  generation on free; `deref` checks the generation matches and panics/errors on
  a stale handle. Catches use-after-free bugs but adds a check per deref.

Avoid a **moving/compacting** collector at first — it requires rewriting every
handle that points at a moved object, which is the hardest part.

### When to run GC
- Simplest: never (until you add it) — just grow the heap.
- Then: trigger when `heap.len()` crosses a threshold that grows with live size
  (e.g. collect when allocations since last GC exceed `2 × live_count`).

---

## 7. Suggested order of work (checklist)

- [ ] Phase A: `ObjRef` + `heap: Vec<Obj>`, migrate strings, all tests green.
- [ ] Move `is_truthy`/`display`/equality to heap-aware VM methods.
- [ ] Decide string equality semantics (identity vs structural / interning).
- [ ] Phase B: `Obj::List` + `MAKE_LIST` + indexing.
- [ ] Phase C: instances / closures as needed.
- [ ] Phase D: mark-sweep GC with a free-list; roots = stack + globals + frames.
- [ ] Switch string constants to preloaded heap handles (Option 2).
- [ ] Add a GC trigger heuristic.

---

## 8. Things to watch

- **Don't hold a `&Obj` across an `alloc`.** `alloc` may grow the `Vec` and
  invalidate the borrow (Rust will likely stop you, but be aware). Read what you
  need, then allocate.
- **GC during allocation.** If `alloc` can trigger GC, make sure any values you
  intend to store are already on the stack / reachable as roots, or they'll be
  collected mid-operation. (This is the classic "GC safe-point" bug.)
- **Keep marks out of `Obj` if you can.** A parallel `Vec<bool>` (or bitset)
  keeps `Obj` smaller and more cache-friendly than embedding a `marked` flag.
