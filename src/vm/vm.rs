use fxhash::FxHashMap;

use crate::Symbol;
use crate::vm::NativeFn;

use super::builtins::builtins;
use super::bytecode::{Bytecode, Constant};
use super::instructions::Instruction;

/// Maximum call-frame depth before a clean "stack overflow" error, so
/// runaway recursion can't exhaust host memory.
const MAX_CALL_DEPTH: usize = 4096;

/// A runtime error: what went wrong, the source line of the failing
/// instruction (0 if unknown), and the axe-level call stack (innermost
/// first) at the moment of the error.
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub line: u32,
    pub trace: Vec<String>,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line != 0 {
            write!(f, "runtime error [line {}]: {}", self.line, self.message)?;
        } else {
            write!(f, "runtime error: {}", self.message)?;
        }
        for entry in &self.trace {
            write!(f, "\n  in {}", entry)?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

/// A heap-allocated object. Owned by the VM's `Heap`, never by a `Value`.
#[derive(Debug, PartialEq)]
pub enum Obj {
    Str(String),
    /// A class: a bag of methods and static members, keyed by interned name,
    /// plus an optional superclass handle for inheritance lookups. Keys are
    /// `Symbol` (interned u32s), so we use `FxHashMap` — SipHash's DoS
    /// resistance is wasted overhead on internal integer keys.
    Class {
        name: Symbol,
        methods: FxHashMap<Symbol, Value>,
        statics: FxHashMap<Symbol, Value>,
        superclass: Option<ObjRef>,
    },
    /// An instance of a class: its own field map plus a handle back to the
    /// class it was created from (for method/static resolution).
    Instance {
        class: ObjRef,
        fields: FxHashMap<Symbol, Value>,
    },
    /// A list of values.
    List(Vec<Value>),
    /// A closure: a function template (entry/arity) plus captured upvalues.
    /// Only *capturing* functions become closures; non-capturing ones stay a
    /// flat `Value::Fn` with no allocation.
    Closure {
        entry: usize,
        arity: u8,
        upvalues: Vec<ObjRef>,
    },
    /// A captured variable. `Open` still lives on the value stack (by absolute
    /// index); `Closed` has been lifted onto the heap once its defining frame
    /// returned, so it outlives that frame.
    Upvalue(UpvalueState),
}

/// State of a captured variable — see `Obj::Upvalue`.
#[derive(Debug, PartialEq)]
pub enum UpvalueState {
    Open(usize),
    Closed(Value),
}

/// A lightweight, `Copy` handle into the `Heap`. Cloning a `Value` that
/// holds one is O(1) — it copies an index, not the underlying object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjRef(usize);

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Obj(ObjRef),
    Native(&'static str, NativeFn),
    Fn { entry: usize, arity: u8 },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Obj(a), Obj(b)) => a == b,
            (Native(a, _), Native(b, _)) => a == b,
            _ => false,
        }
    }
}

impl Value {
    fn as_bool(&self, heap: &Heap) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Obj(o) => match heap.get(*o) {
                Obj::Str(s) => !s.is_empty(),
                Obj::List(items) => !items.is_empty(),
                Obj::Class { .. }
                | Obj::Instance { .. }
                | Obj::Closure { .. }
                | Obj::Upvalue(_) => true,
            },
            Value::Native(_, _) => true,
            Value::Fn { .. } => true,
        }
    }

    fn is_truthy(&self, heap: &Heap) -> bool {
        self.as_bool(heap)
    }

    pub fn display(&self, heap: &Heap) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Obj(o) => match heap.get(*o) {
                Obj::Str(s) => s.clone(),
                Obj::Class { .. } => "<class>".to_string(),
                Obj::Instance { .. } => "<instance>".to_string(),
                Obj::List(items) => {
                    let inner: Vec<String> = items.iter().map(|v| v.display(heap)).collect();
                    format!("[{}]", inner.join(", "))
                }
                Obj::Closure { entry, arity, .. } => format!("<closure @{} /{}>", entry, arity),
                Obj::Upvalue(_) => "<upvalue>".to_string(),
            },
            Value::Native(name, _) => format!("<native-fn {}>", name),
            Value::Fn { entry, arity } => format!("<fn @{} /{}>", entry, arity),
        }
    }
}

/// GC kicks in once this many objects are live (and adapts from there).
const INITIAL_GC_THRESHOLD: usize = 1024;

/// The VM-owned object store. All heap objects live here; `Value`s only
/// reference them through `ObjRef` handles.
///
/// Collection is mark-sweep and non-moving: dead slots become `None` and go
/// on a free list for reuse, so live `ObjRef` handles are never invalidated.
pub struct Heap {
    objects: Vec<Option<Obj>>,
    /// Indices of dead slots available for reuse.
    free: Vec<usize>,
    /// Number of occupied slots.
    live: usize,
    /// Collect when `live` reaches this. Doubled from the survivor count
    /// after each collection so GC cost stays proportional to live data.
    next_gc: usize,
}

impl Heap {
    fn new() -> Self {
        Heap {
            objects: Vec::new(),
            free: Vec::new(),
            live: 0,
            next_gc: INITIAL_GC_THRESHOLD,
        }
    }

    /// Whether enough objects are live that the VM should collect at the
    /// next safepoint (before its next allocation).
    fn should_collect(&self) -> bool {
        self.live >= self.next_gc
    }

    /// Allocate an object and return a handle to it, reusing a dead slot
    /// when one is available.
    fn alloc(&mut self, obj: Obj) -> ObjRef {
        self.live += 1;
        match self.free.pop() {
            Some(index) => {
                self.objects[index] = Some(obj);
                ObjRef(index)
            }
            None => {
                let index = self.objects.len();
                self.objects.push(Some(obj));
                ObjRef(index)
            }
        }
    }

    /// Allocate a string object and wrap its handle in a `Value`.
    fn alloc_str(&mut self, s: impl Into<String>) -> Value {
        Value::Obj(self.alloc(Obj::Str(s.into())))
    }

    /// Allocate an empty class object and wrap its handle in a `Value`.
    fn alloc_class(&mut self, name: Symbol) -> Value {
        Value::Obj(self.alloc(Obj::Class {
            name,
            methods: FxHashMap::default(),
            statics: FxHashMap::default(),
            superclass: None,
        }))
    }

    /// Allocate an instance of `class` with no fields yet.
    fn alloc_instance(&mut self, class: ObjRef) -> Value {
        Value::Obj(self.alloc(Obj::Instance {
            class,
            fields: FxHashMap::default(),
        }))
    }

    /// Allocate a list object and wrap its handle in a `Value`. Public so
    /// native functions (e.g. `range`) can build lists.
    pub fn alloc_list(&mut self, items: Vec<Value>) -> Value {
        Value::Obj(self.alloc(Obj::List(items)))
    }

    /// Allocate a closure object and wrap its handle in a `Value`.
    fn alloc_closure(&mut self, entry: usize, arity: u8, upvalues: Vec<ObjRef>) -> Value {
        Value::Obj(self.alloc(Obj::Closure {
            entry,
            arity,
            upvalues,
        }))
    }

    /// Allocate an open upvalue pointing at absolute stack index `idx`.
    fn alloc_upvalue(&mut self, idx: usize) -> ObjRef {
        self.alloc(Obj::Upvalue(UpvalueState::Open(idx)))
    }

    /// Length of a list or string value. Public for the `len` native function.
    pub fn value_len(&self, value: &Value) -> Result<i64, String> {
        match value {
            Value::Obj(o) => match self.get(*o) {
                Obj::List(items) => Ok(items.len() as i64),
                Obj::Str(s) => Ok(s.chars().count() as i64),
                _ => Err("value has no length".to_string()),
            },
            _ => Err("value has no length".to_string()),
        }
    }

    /// Dereference a handle to the object it points at.
    fn get(&self, r: ObjRef) -> &Obj {
        self.objects[r.0].as_ref().expect("use after free")
    }

    /// Mutably dereference a handle to the object it points at.
    fn get_mut(&mut self, r: ObjRef) -> &mut Obj {
        self.objects[r.0].as_mut().expect("use after free")
    }

    /// Mark `r` (if unmarked) and queue it for tracing.
    fn mark_ref(r: ObjRef, marks: &mut [bool], gray: &mut Vec<ObjRef>) {
        if !marks[r.0] {
            marks[r.0] = true;
            gray.push(r);
        }
    }

    /// Mark the object a value references, if any.
    fn mark_value(v: &Value, marks: &mut [bool], gray: &mut Vec<ObjRef>) {
        if let Value::Obj(r) = v {
            Self::mark_ref(*r, marks, gray);
        }
    }

    /// Trace every reference held by the (already marked) object `r`.
    fn trace(&self, r: ObjRef, marks: &mut [bool], gray: &mut Vec<ObjRef>) {
        match self.get(r) {
            Obj::Str(_) => {}
            Obj::Class {
                methods,
                statics,
                superclass,
                ..
            } => {
                for v in methods.values().chain(statics.values()) {
                    Self::mark_value(v, marks, gray);
                }
                if let Some(s) = superclass {
                    Self::mark_ref(*s, marks, gray);
                }
            }
            Obj::Instance { class, fields } => {
                Self::mark_ref(*class, marks, gray);
                for v in fields.values() {
                    Self::mark_value(v, marks, gray);
                }
            }
            Obj::List(items) => {
                for v in items {
                    Self::mark_value(v, marks, gray);
                }
            }
            Obj::Closure { upvalues, .. } => {
                for uv in upvalues {
                    Self::mark_ref(*uv, marks, gray);
                }
            }
            // Open upvalues point into the value stack, which is a root
            // itself; only closed ones own a value to trace.
            Obj::Upvalue(UpvalueState::Open(_)) => {}
            Obj::Upvalue(UpvalueState::Closed(v)) => Self::mark_value(v, marks, gray),
        }
    }

    /// Free every unmarked object, returning its slot to the free list.
    fn sweep(&mut self, marks: &[bool]) {
        for (i, slot) in self.objects.iter_mut().enumerate() {
            if slot.is_some() && !marks[i] {
                *slot = None;
                self.free.push(i);
                self.live -= 1;
            }
        }
        self.next_gc = (self.live * 2).max(INITIAL_GC_THRESHOLD);
    }

    /// Look up a method by name, walking the superclass chain. Returns a clone
    /// of the stored `Value` (a `Value::Fn`). `None` if `class` isn't a class
    /// or no ancestor defines the method.
    fn find_method(&self, class: ObjRef, name: Symbol) -> Option<Value> {
        let mut cur = Some(class);
        while let Some(c) = cur {
            match self.get(c) {
                Obj::Class {
                    methods,
                    superclass,
                    ..
                } => {
                    if let Some(v) = methods.get(&name) {
                        return Some(v.clone());
                    }
                    cur = *superclass;
                }
                _ => return None,
            }
        }
        None
    }

    /// Look up a static member by name, walking the superclass chain.
    fn find_static(&self, class: ObjRef, name: Symbol) -> Option<Value> {
        let mut cur = Some(class);
        while let Some(c) = cur {
            match self.get(c) {
                Obj::Class {
                    statics,
                    superclass,
                    ..
                } => {
                    if let Some(v) = statics.get(&name) {
                        return Some(v.clone());
                    }
                    cur = *superclass;
                }
                _ => return None,
            }
        }
        None
    }
}

struct Frame {
    ret_ip: usize,
    bp: usize,
    return_override: Option<Value>,
    closure: usize,
    /// Bytecode entry of the function this frame is executing, for naming
    /// the frame in stack traces.
    entry: usize,
}

const NO_CLOSURE: usize = usize::MAX;

pub struct AxeVM<'a> {
    bytecode: &'a Bytecode,
    ip: usize,
    bp: usize,
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: Vec<Value>,
    heap: Heap,
    open_upvalues: Vec<ObjRef>,
    str_constants: Vec<Option<ObjRef>>,
    /// When set (env `AXE_GC_STRESS=1`), collect at every safepoint — slow,
    /// but shakes out objects the collector wrongly considers unreachable.
    gc_stress: bool,
    /// Offset of the opcode currently being executed, so errors can be
    /// attributed to the right instruction (and thus source line).
    op_ip: usize,
}

impl<'a> AxeVM<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        let globals = builtins()
            .iter()
            .map(|(name, f)| Value::Native(name, *f))
            .collect();

        AxeVM {
            bytecode,
            ip: 0,
            bp: 0,
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(256),
            globals,
            heap: Heap::new(),
            open_upvalues: Vec::new(),
            str_constants: vec![None; bytecode.constants.len()],
            gc_stress: std::env::var_os("AXE_GC_STRESS").is_some(),
            op_ip: 0,
        }
    }

    /// Build a `RuntimeError` at the current instruction, with a stack trace.
    #[cold]
    fn rt_err(&self, message: impl Into<String>) -> RuntimeError {
        let mut full: Vec<String> = self
            .frames
            .iter()
            .rev()
            .map(|frame| {
                let name = self.bytecode.fn_name(frame.entry).unwrap_or("<fn>");
                let call_line = self.bytecode.line_at(frame.ret_ip.saturating_sub(1));
                if call_line != 0 {
                    format!("{} (called from line {})", name, call_line)
                } else {
                    name.to_string()
                }
            })
            .collect();
        // Deep traces (e.g. stack overflow) get elided in the middle.
        let trace = if full.len() > 16 {
            let omitted = full.len() - 12;
            let tail = full.split_off(full.len() - 2);
            full.truncate(10);
            full.push(format!("... {} frames omitted ...", omitted));
            full.extend(tail);
            full
        } else {
            full
        };
        RuntimeError {
            message: message.into(),
            line: self.bytecode.line_at(self.op_ip),
            trace,
        }
    }

    /// Human-readable type of a value, for error messages.
    fn type_name(&self, v: &Value) -> &'static str {
        match v {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Native(..) | Value::Fn { .. } => "function",
            Value::Obj(r) => match self.heap.get(*r) {
                Obj::Str(_) => "string",
                Obj::List(_) => "list",
                Obj::Class { .. } => "class",
                Obj::Instance { .. } => "instance",
                Obj::Closure { .. } => "function",
                Obj::Upvalue(_) => "upvalue",
            },
        }
    }

    /// Type error for a binary operator applied to unsupported operands.
    #[cold]
    fn binop_err(&self, op: &str, a: &Value, b: &Value) -> RuntimeError {
        self.rt_err(format!(
            "unsupported operand types for {}: {} and {}",
            op,
            self.type_name(a),
            self.type_name(b)
        ))
    }

    /// Pop a value that must be an int (bitwise ops).
    fn pop_int(&mut self, op: &str) -> Result<i64, RuntimeError> {
        match self.pop() {
            Value::Int(n) => Ok(n),
            v => Err(self.rt_err(format!(
                "unsupported operand type for {}: {}",
                op,
                self.type_name(&v)
            ))),
        }
    }

    /// Guard against runaway recursion before pushing a call frame.
    fn check_depth(&self) -> Result<(), RuntimeError> {
        if self.frames.len() >= MAX_CALL_DEPTH {
            return Err(self.rt_err("stack overflow"));
        }
        Ok(())
    }

    /// Error for property access on something that isn't an instance.
    #[cold]
    fn property_target_err(&self, name: Symbol, target: &Value) -> RuntimeError {
        self.rt_err(format!(
            "cannot access property '{}' on {}",
            self.bytecode.sym_name(name),
            self.type_name(target)
        ))
    }

    /// Error for a method call on something that isn't an instance.
    #[cold]
    fn method_target_err(&self, name: Symbol, target: &Value) -> RuntimeError {
        self.rt_err(format!(
            "cannot call method '{}' on {}",
            self.bytecode.sym_name(name),
            self.type_name(target)
        ))
    }

    /// Verify a call's argument count matches the callee's arity.
    fn arity_check(&self, entry: usize, arity: usize, argc: usize) -> Result<(), RuntimeError> {
        if arity != argc {
            let name = self.bytecode.fn_name(entry).unwrap_or("<fn>");
            return Err(self.rt_err(format!(
                "{} expects {} argument{} but got {}",
                name,
                arity,
                if arity == 1 { "" } else { "s" },
                argc
            )));
        }
        Ok(())
    }

    /// GC safepoint: collect if the heap has grown past its threshold. Called
    /// right before allocating opcodes touch the heap, while every live value
    /// is still reachable from a root.
    fn maybe_gc(&mut self) {
        if self.heap.should_collect() || self.gc_stress {
            self.collect_garbage();
        }
    }

    /// Mark-sweep collection. Roots: the value stack, globals, call frames
    /// (their closures and pending `return_override`s), open upvalues, and
    /// the interned string constants (pinned for the life of the VM).
    fn collect_garbage(&mut self) {
        let mut marks = vec![false; self.heap.objects.len()];
        let mut gray: Vec<ObjRef> = Vec::new();

        for v in &self.stack {
            Heap::mark_value(v, &mut marks, &mut gray);
        }
        for v in &self.globals {
            Heap::mark_value(v, &mut marks, &mut gray);
        }
        for frame in &self.frames {
            if frame.closure != NO_CLOSURE {
                Heap::mark_ref(ObjRef(frame.closure), &mut marks, &mut gray);
            }
            if let Some(v) = &frame.return_override {
                Heap::mark_value(v, &mut marks, &mut gray);
            }
        }
        for &uv in &self.open_upvalues {
            Heap::mark_ref(uv, &mut marks, &mut gray);
        }
        for handle in self.str_constants.iter().flatten() {
            Heap::mark_ref(*handle, &mut marks, &mut gray);
        }

        while let Some(r) = gray.pop() {
            self.heap.trace(r, &mut marks, &mut gray);
        }

        self.heap.sweep(&marks);
    }

    fn capture_upvalue(&mut self, idx: usize) -> ObjRef {
        for &uv in &self.open_upvalues {
            if let Obj::Upvalue(UpvalueState::Open(i)) = self.heap.get(uv)
                && *i == idx
            {
                return uv;
            }
        }
        let uv = self.heap.alloc_upvalue(idx);
        self.open_upvalues.push(uv);
        uv
    }

    fn current_upvalue(&self, slot: usize) -> ObjRef {
        let closure = self.frames.last().map(|f| f.closure).unwrap_or(NO_CLOSURE);
        assert_ne!(closure, NO_CLOSURE, "upvalue access outside a closure");
        match self.heap.get(ObjRef(closure)) {
            Obj::Closure { upvalues, .. } => upvalues[slot],
            _ => unreachable!("frame closure is not a closure"),
        }
    }

    fn close_upvalues(&mut self, from: usize) {
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let uv = self.open_upvalues[i];
            let idx = match self.heap.get(uv) {
                Obj::Upvalue(UpvalueState::Open(idx)) => *idx,
                _ => {
                    self.open_upvalues.swap_remove(i);
                    continue;
                }
            };
            if idx >= from {
                let value = self.stack[idx].clone();
                if let Obj::Upvalue(state) = self.heap.get_mut(uv) {
                    *state = UpvalueState::Closed(value);
                }
                self.open_upvalues.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Render a value as a display string, resolving heap objects.
    pub fn display_value(&self, value: &Value) -> String {
        value.display(&self.heap)
    }

    /// Execute the bytecode from the top. On error, the VM state is reset on
    /// the next `exec` call, so a REPL can keep using the same VM.
    pub fn exec(&mut self) -> Result<Option<Value>, RuntimeError> {
        self.ip = 0;
        self.bp = 0;
        self.stack.clear();
        self.frames.clear();
        self.open_upvalues.clear();
        self.eval()?;
        Ok(self.stack.pop())
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack underflow")
    }

    fn peek(&self) -> &Value {
        self.stack.last().expect("Stack underflow")
    }

    fn read_u8(&mut self) -> u8 {
        let value = self.bytecode.code[self.ip];
        self.ip += 1;
        value
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_u8() as usize;
        // Copy the shared bytecode reference so the match borrow isn't tied to
        // `&self`, leaving `self.heap` / `self.str_constants` free to mutate.
        let bytecode = self.bytecode;
        match &bytecode.constants[index] {
            Constant::Int(n) => Value::Int(*n),
            Constant::Float(n) => Value::Float(*n),
            Constant::Fn { entry, arity } => Value::Fn {
                entry: *entry,
                arity: *arity,
            },
            Constant::Str(s) => match self.str_constants[index] {
                // Immutable string constant already on the heap — reuse its handle.
                Some(handle) => Value::Obj(handle),
                None => {
                    let value = self.heap.alloc_str(s.clone());
                    if let Value::Obj(handle) = value {
                        self.str_constants[index] = Some(handle);
                    }
                    value
                }
            },
            Constant::Sym(_) => panic!("symbol constant cannot be loaded as a value"),
        }
    }

    /// Read a u8 operand indexing a `Constant::Sym` and return the `Symbol`.
    /// Used by the OO opcodes whose operand is a member name.
    fn read_sym(&mut self) -> Symbol {
        let index = self.read_u8() as usize;
        match self.bytecode.constants[index] {
            Constant::Sym(s) => s,
            ref other => panic!("expected symbol constant, got {:?}", other),
        }
    }

    fn read_u16(&mut self) -> u16 {
        let lo = self.bytecode.code[self.ip];
        let hi = self.bytecode.code[self.ip + 1];
        self.ip += 2;
        u16::from_le_bytes([lo, hi])
    }

    fn eval(&mut self) -> Result<(), RuntimeError> {
        loop {
            self.op_ip = self.ip;
            let opcode = self.read_u8();
            match opcode {
                Instruction::HALT => break,

                Instruction::JUMP => {
                    let offset = self.read_u16() as usize;
                    self.ip += offset;
                }

                Instruction::JUMP_IF_FALSE => {
                    let offset = self.read_u16() as usize;
                    let cond = self.pop();
                    if !cond.is_truthy(&self.heap) {
                        self.ip += offset;
                    }
                }

                Instruction::LOOP => {
                    let offset = self.read_u16() as usize;
                    self.ip -= offset;
                }

                // Stack operations
                Instruction::CONST => {
                    let value = self.read_constant();
                    self.push(value);
                }

                Instruction::POP => {
                    self.pop();
                }

                Instruction::DUP => {
                    let value = self.peek().clone();
                    self.push(value);
                }

                // Literals
                Instruction::NULL => self.push(Value::Null),
                Instruction::TRUE => self.push(Value::Bool(true)),
                Instruction::FALSE => self.push(Value::Bool(false)),

                // Arithmetic
                Instruction::ADD => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(
                            a.checked_add(*b)
                                .ok_or_else(|| self.rt_err("integer overflow in +"))?,
                        ),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                        (Value::Obj(ao), Value::Obj(bo)) => {
                            let s = match (self.heap.get(*ao), self.heap.get(*bo)) {
                                (Obj::Str(a), Obj::Str(b)) => format!("{}{}", a, b),
                                _ => return Err(self.binop_err("+", &a, &b)),
                            };
                            // Safepoint: operands are already folded into `s`,
                            // so nothing this alloc needs can be collected.
                            self.maybe_gc();
                            self.heap.alloc_str(s)
                        }
                        _ => return Err(self.binop_err("+", &a, &b)),
                    };
                    self.push(result);
                }

                Instruction::SUB => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(
                            a.checked_sub(*b)
                                .ok_or_else(|| self.rt_err("integer overflow in -"))?,
                        ),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        _ => return Err(self.binop_err("-", &a, &b)),
                    };
                    self.push(result);
                }

                Instruction::MUL => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(
                            a.checked_mul(*b)
                                .ok_or_else(|| self.rt_err("integer overflow in *"))?,
                        ),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                        _ => return Err(self.binop_err("*", &a, &b)),
                    };
                    self.push(result);
                }

                Instruction::DIV => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => {
                            Value::Int(a.checked_div(*b).ok_or_else(|| {
                                if *b == 0 {
                                    self.rt_err("division by zero")
                                } else {
                                    self.rt_err("integer overflow in /")
                                }
                            })?)
                        }
                        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                        _ => return Err(self.binop_err("/", &a, &b)),
                    };
                    self.push(result);
                }

                Instruction::MOD => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => {
                            Value::Int(a.checked_rem(*b).ok_or_else(|| {
                                if *b == 0 {
                                    self.rt_err("division by zero in %")
                                } else {
                                    self.rt_err("integer overflow in %")
                                }
                            })?)
                        }
                        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                        _ => return Err(self.binop_err("%", &a, &b)),
                    };
                    self.push(result);
                }

                Instruction::NEG => {
                    let a = self.pop();
                    let result = match a {
                        Value::Int(n) => Value::Int(
                            n.checked_neg()
                                .ok_or_else(|| self.rt_err("integer overflow in negation"))?,
                        ),
                        Value::Float(n) => Value::Float(-n),
                        _ => {
                            return Err(self.rt_err(format!(
                                "unsupported operand type for unary -: {}",
                                self.type_name(&a)
                            )));
                        }
                    };
                    self.push(result);
                }

                // Comparison
                Instruction::EQ => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }

                Instruction::NEQ => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }

                Instruction::LT => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => a < b,
                        (Value::Float(a), Value::Float(b)) => a < b,
                        _ => return Err(self.binop_err("<", &a, &b)),
                    };
                    self.push(Value::Bool(result));
                }

                Instruction::LTE => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => a <= b,
                        (Value::Float(a), Value::Float(b)) => a <= b,
                        _ => return Err(self.binop_err("<=", &a, &b)),
                    };
                    self.push(Value::Bool(result));
                }

                Instruction::GT => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => a > b,
                        (Value::Float(a), Value::Float(b)) => a > b,
                        _ => return Err(self.binop_err(">", &a, &b)),
                    };
                    self.push(Value::Bool(result));
                }

                Instruction::GTE => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => a >= b,
                        (Value::Float(a), Value::Float(b)) => a >= b,
                        _ => return Err(self.binop_err(">=", &a, &b)),
                    };
                    self.push(Value::Bool(result));
                }

                // Logical
                Instruction::NOT => {
                    let a = self.pop();
                    let result = !a.is_truthy(&self.heap);
                    self.push(Value::Bool(result));
                }

                Instruction::AND => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = a.is_truthy(&self.heap) && b.is_truthy(&self.heap);
                    self.push(Value::Bool(result));
                }

                Instruction::OR => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = a.is_truthy(&self.heap) || b.is_truthy(&self.heap);
                    self.push(Value::Bool(result));
                }

                // Bitwise
                Instruction::BITAND => {
                    let b = self.pop_int("&")?;
                    let a = self.pop_int("&")?;
                    self.push(Value::Int(a & b));
                }

                Instruction::BITOR => {
                    let b = self.pop_int("|")?;
                    let a = self.pop_int("|")?;
                    self.push(Value::Int(a | b));
                }

                Instruction::BITINV => {
                    let a = self.pop_int("~")?;
                    self.push(Value::Int(!a));
                }

                Instruction::DEFINE_GLOBAL => {
                    let idx = self.read_u8() as usize;
                    let value = self.pop();
                    if idx >= self.globals.len() {
                        self.globals.resize(idx + 1, Value::Null)
                    }
                    self.globals[idx] = value;
                }
                Instruction::GET_GLOBAL => {
                    let idx = self.read_u8() as usize;
                    self.push(self.globals[idx].clone());
                }

                Instruction::SET_GLOBAL => {
                    let idx = self.read_u8() as usize;
                    self.globals[idx] = self.peek().clone();
                }

                Instruction::DEFINE_LOCAL => {
                    let slot = self.read_u8() as usize;
                    let value = self.peek().clone();
                    self.stack[self.bp + slot] = value;
                }

                Instruction::SET_LOCAL => {
                    let slot = self.read_u8() as usize;
                    let value = self.peek().clone();
                    self.stack[self.bp + slot] = value;
                }

                Instruction::GET_LOCAL => {
                    let slot = self.read_u8() as usize;
                    let value = self.stack[self.bp + slot].clone();
                    self.push(value);
                }
                Instruction::CALL => {
                    let argc = self.read_u8() as usize;
                    let callee_idx = self.stack.len() - argc - 1;
                    let callee = self.stack[callee_idx].clone();
                    match callee {
                        Value::Native(name, func) => {
                            let args: Vec<Value> = self.stack[callee_idx + 1..].to_vec();
                            let result = match func(&args, &mut self.heap) {
                                Ok(v) => v,
                                Err(m) => return Err(self.rt_err(format!("{}: {}", name, m))),
                            };
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::Fn { entry, arity } => {
                            self.arity_check(entry, arity as usize, argc)?;
                            self.check_depth()?;
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: NO_CLOSURE,
                                entry,
                            });
                            self.bp = callee_idx + 1;
                            self.ip = entry;
                        }
                        Value::Obj(closure_ref) => {
                            let (entry, arity) = match self.heap.get(closure_ref) {
                                Obj::Closure { entry, arity, .. } => (*entry, *arity),
                                _ => {
                                    return Err(self.rt_err(format!(
                                        "{} is not callable",
                                        self.type_name(&callee)
                                    )));
                                }
                            };
                            self.arity_check(entry, arity as usize, argc)?;
                            self.check_depth()?;
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: closure_ref.0,
                                entry,
                            });
                            self.bp = callee_idx + 1;
                            self.ip = entry;
                        }
                        other => {
                            return Err(
                                self.rt_err(format!("{} is not callable", self.type_name(&other)))
                            );
                        }
                    }
                }
                Instruction::RETURN => {
                    let result = self.pop();
                    let frame = self.frames.pop().expect("return outside function");
                    // Close any upvalues that captured this frame's locals before
                    // they're torn off the stack. Guarded so the common no-closure
                    // path pays only a branch, not a call.
                    if !self.open_upvalues.is_empty() {
                        self.close_upvalues(self.bp);
                    }
                    self.stack.truncate(self.bp - 1);
                    self.ip = frame.ret_ip;
                    self.bp = frame.bp;
                    self.push(frame.return_override.unwrap_or(result));
                }

                Instruction::CLASS => {
                    let name = self.read_sym();
                    self.maybe_gc();
                    let class = self.heap.alloc_class(name);
                    self.push(class);
                }

                Instruction::INHERIT => {
                    let superclass = self.pop();
                    let class = self.peek().clone();
                    let (Value::Obj(class_ref), Value::Obj(super_ref)) =
                        (class, superclass.clone())
                    else {
                        return Err(self.rt_err(format!(
                            "can only inherit from a class, got {}",
                            self.type_name(&superclass)
                        )));
                    };
                    if !matches!(self.heap.get(super_ref), Obj::Class { .. }) {
                        return Err(self.rt_err(format!(
                            "can only inherit from a class, got {}",
                            self.type_name(&superclass)
                        )));
                    }
                    if let Obj::Class { superclass, .. } = self.heap.get_mut(class_ref) {
                        *superclass = Some(super_ref);
                    } else {
                        panic!("INHERIT target is not a class");
                    }
                }

                Instruction::METHOD => {
                    let name = self.read_sym();
                    let method = self.pop();
                    let Value::Obj(class_ref) = self.peek().clone() else {
                        panic!("METHOD target is not a class");
                    };
                    if let Obj::Class { methods, .. } = self.heap.get_mut(class_ref) {
                        methods.insert(name, method);
                    } else {
                        panic!("METHOD target is not a class");
                    }
                }

                Instruction::STATIC_FIELD => {
                    let name = self.read_sym();
                    let value = self.pop();
                    let Value::Obj(class_ref) = self.peek().clone() else {
                        panic!("STATIC_FIELD target is not a class");
                    };
                    if let Obj::Class { statics, .. } = self.heap.get_mut(class_ref) {
                        statics.insert(name, value);
                    } else {
                        panic!("STATIC_FIELD target is not a class");
                    }
                }

                Instruction::GET_PROPERTY => {
                    let name = self.read_sym();
                    let target = self.pop();
                    let obj_ref = match target {
                        Value::Obj(r) => r,
                        _ => return Err(self.property_target_err(name, &target)),
                    };
                    let (field, class) = match self.heap.get(obj_ref) {
                        Obj::Instance { fields, class } => (fields.get(&name).cloned(), *class),
                        _ => return Err(self.property_target_err(name, &target)),
                    };
                    let value = field
                        .or_else(|| self.heap.find_method(class, name))
                        .or_else(|| self.heap.find_static(class, name))
                        .ok_or_else(|| {
                            self.rt_err(format!(
                                "undefined property '{}'",
                                self.bytecode.sym_name(name)
                            ))
                        })?;
                    self.push(value);
                }

                Instruction::SET_PROPERTY => {
                    let name = self.read_sym();
                    let value = self.pop();
                    let target = self.pop();
                    let obj_ref = match target {
                        Value::Obj(r) => r,
                        _ => return Err(self.property_target_err(name, &target)),
                    };
                    if let Obj::Instance { fields, .. } = self.heap.get_mut(obj_ref) {
                        fields.insert(name, value.clone());
                    } else {
                        return Err(self.property_target_err(name, &target));
                    }
                    self.push(value);
                }

                Instruction::GET_STATIC => {
                    let name = self.read_sym();
                    let target = self.pop();
                    let Value::Obj(class_ref) = target else {
                        return Err(self.rt_err(format!(
                            "cannot access static member '{}' on {}",
                            self.bytecode.sym_name(name),
                            self.type_name(&target)
                        )));
                    };
                    let value = self
                        .heap
                        .find_static(class_ref, name)
                        .or_else(|| self.heap.find_method(class_ref, name))
                        .ok_or_else(|| {
                            self.rt_err(format!(
                                "undefined static member '{}'",
                                self.bytecode.sym_name(name)
                            ))
                        })?;
                    self.push(value);
                }

                Instruction::NEW => {
                    let init_name = self.read_sym();
                    let argc = self.read_u8() as usize;
                    let class_idx = self.stack.len() - argc - 1;
                    let class_val = self.stack[class_idx].clone();
                    let Value::Obj(class_ref) = class_val else {
                        return Err(self.rt_err(format!(
                            "can only 'new' a class, got {}",
                            self.type_name(&class_val)
                        )));
                    };
                    if !matches!(self.heap.get(class_ref), Obj::Class { .. }) {
                        return Err(self.rt_err(format!(
                            "can only 'new' a class, got {}",
                            self.type_name(&class_val)
                        )));
                    }
                    // Safepoint: class and args are still rooted on the stack.
                    self.maybe_gc();
                    let instance = self.heap.alloc_instance(class_ref);

                    match self.heap.find_method(class_ref, init_name) {
                        Some(Value::Fn { entry, arity }) => {
                            // init receives (self, args...): arity counts self.
                            if arity as usize != argc + 1 {
                                return Err(self.rt_err(format!(
                                    "init expects {} argument(s) but got {}",
                                    arity - 1,
                                    argc
                                )));
                            }
                            self.check_depth()?;
                            // Reshape [class, args..] into [init_fn, self, args..] so
                            // the call reuses the standard frame layout, and stash the
                            // instance so RETURN yields it instead of init's result.
                            self.stack[class_idx] = Value::Fn { entry, arity };
                            self.stack.insert(class_idx + 1, instance.clone());
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: Some(instance),
                                closure: NO_CLOSURE,
                                entry,
                            });
                            self.bp = class_idx + 1;
                            self.ip = entry;
                        }
                        None => {
                            // No constructor: discard args, yield the bare instance.
                            self.stack.truncate(class_idx);
                            self.push(instance);
                        }
                        Some(_) => {
                            return Err(self.rt_err("init is not a function"));
                        }
                    }
                }

                Instruction::INVOKE => {
                    let name = self.read_sym();
                    let argc = self.read_u8() as usize;
                    let recv_idx = self.stack.len() - argc - 1;
                    let recv = self.stack[recv_idx].clone();
                    let Value::Obj(obj_ref) = recv else {
                        return Err(self.method_target_err(name, &recv));
                    };
                    let class = match self.heap.get(obj_ref) {
                        Obj::Instance { class, .. } => *class,
                        _ => return Err(self.method_target_err(name, &recv)),
                    };
                    match self.heap.find_method(class, name) {
                        Some(Value::Fn { entry, arity }) => {
                            // method receives (self, args...): arity counts self.
                            if arity as usize != argc + 1 {
                                return Err(self.rt_err(format!(
                                    "{} expects {} argument(s) but got {}",
                                    self.bytecode.sym_name(name),
                                    arity - 1,
                                    argc
                                )));
                            }
                            self.check_depth()?;
                            // Insert the callee below the receiver so the receiver
                            // becomes slot 0 (self) of the new frame.
                            self.stack.insert(recv_idx, Value::Fn { entry, arity });
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: NO_CLOSURE,
                                entry,
                            });
                            self.bp = recv_idx + 1;
                            self.ip = entry;
                        }
                        _ => {
                            return Err(self.rt_err(format!(
                                "undefined method '{}'",
                                self.bytecode.sym_name(name)
                            )));
                        }
                    }
                }

                Instruction::STATIC_INVOKE => {
                    let name = self.read_sym();
                    let argc = self.read_u8() as usize;
                    let class_idx = self.stack.len() - argc - 1;
                    let class_val = self.stack[class_idx].clone();
                    let Value::Obj(class_ref) = class_val else {
                        return Err(self.rt_err(format!(
                            "cannot call static method '{}' on {}",
                            self.bytecode.sym_name(name),
                            self.type_name(&class_val)
                        )));
                    };
                    let method = self
                        .heap
                        .find_method(class_ref, name)
                        .or_else(|| self.heap.find_static(class_ref, name));
                    match method {
                        Some(Value::Fn { entry, arity }) => {
                            if arity as usize != argc {
                                return Err(self.rt_err(format!(
                                    "{} expects {} argument(s) but got {}",
                                    self.bytecode.sym_name(name),
                                    arity,
                                    argc
                                )));
                            }
                            self.check_depth()?;
                            // Replace the class with the callee; args are slots 0..
                            self.stack[class_idx] = Value::Fn { entry, arity };
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: NO_CLOSURE,
                                entry,
                            });
                            self.bp = class_idx + 1;
                            self.ip = entry;
                        }
                        _ => {
                            return Err(self.rt_err(format!(
                                "undefined static method '{}'",
                                self.bytecode.sym_name(name)
                            )));
                        }
                    }
                }

                Instruction::BUILD_LIST => {
                    let count = self.read_u8() as usize;
                    // Safepoint: the elements are still rooted on the stack.
                    self.maybe_gc();
                    let start = self.stack.len() - count;
                    let items: Vec<Value> = self.stack.split_off(start);
                    let list = self.heap.alloc_list(items);
                    self.push(list);
                }

                Instruction::GET_INDEX => {
                    let index = self.pop();
                    let list = self.pop();
                    let idx = match index {
                        Value::Int(n) => n,
                        other => {
                            return Err(self.rt_err(format!(
                                "list index must be an int, got {}",
                                self.type_name(&other)
                            )));
                        }
                    };
                    let element = match &list {
                        Value::Obj(obj_ref) => match self.heap.get(*obj_ref) {
                            Obj::List(items) => {
                                let len = items.len() as i64;
                                let resolved = if idx < 0 { idx + len } else { idx };
                                if resolved < 0 || resolved >= len {
                                    return Err(self.rt_err(format!(
                                        "list index {} out of bounds (length {})",
                                        idx, len
                                    )));
                                }
                                items[resolved as usize].clone()
                            }
                            _ => {
                                return Err(
                                    self.rt_err(format!("cannot index {}", self.type_name(&list)))
                                );
                            }
                        },
                        _ => {
                            return Err(
                                self.rt_err(format!("cannot index {}", self.type_name(&list)))
                            );
                        }
                    };
                    self.push(element);
                }

                Instruction::LEN => {
                    let value = self.pop();
                    let len = match self.heap.value_len(&value) {
                        Ok(n) => n,
                        Err(m) => return Err(self.rt_err(m)),
                    };
                    self.push(Value::Int(len));
                }

                Instruction::CLOSURE => {
                    // Safepoint up front: the upvalues captured below stay
                    // reachable via `open_upvalues` / the enclosing closure,
                    // and no further collection can occur mid-handler.
                    self.maybe_gc();
                    let (entry, arity) = match self.read_constant() {
                        Value::Fn { entry, arity } => (entry, arity),
                        other => panic!("CLOSURE expects a function constant, got {:?}", other),
                    };
                    let count = self.read_u8() as usize;
                    let mut upvalues = Vec::with_capacity(count);
                    for _ in 0..count {
                        let is_local = self.read_u8() != 0;
                        let index = self.read_u8() as usize;
                        if is_local {
                            // Capture a local of the enclosing (currently running) frame.
                            let abs = self.bp + index;
                            upvalues.push(self.capture_upvalue(abs));
                        } else {
                            // Inherit an upvalue from the enclosing closure.
                            let enclosing =
                                self.frames.last().map(|f| f.closure).unwrap_or(NO_CLOSURE);
                            assert_ne!(
                                enclosing, NO_CLOSURE,
                                "non-local upvalue capture outside an enclosing closure"
                            );
                            let uv = match self.heap.get(ObjRef(enclosing)) {
                                Obj::Closure { upvalues, .. } => upvalues[index],
                                _ => unreachable!("enclosing closure is not a closure"),
                            };
                            upvalues.push(uv);
                        }
                    }
                    let closure = self.heap.alloc_closure(entry, arity, upvalues);
                    self.push(closure);
                }

                Instruction::GET_UPVALUE => {
                    let slot = self.read_u8() as usize;
                    let uv = self.current_upvalue(slot);
                    let value = match self.heap.get(uv) {
                        Obj::Upvalue(UpvalueState::Open(idx)) => self.stack[*idx].clone(),
                        Obj::Upvalue(UpvalueState::Closed(v)) => v.clone(),
                        _ => unreachable!("upvalue slot is not an upvalue"),
                    };
                    self.push(value);
                }

                Instruction::SET_UPVALUE => {
                    let slot = self.read_u8() as usize;
                    let value = self.peek().clone();
                    let uv = self.current_upvalue(slot);
                    let open_idx = match self.heap.get(uv) {
                        Obj::Upvalue(UpvalueState::Open(idx)) => Some(*idx),
                        _ => None,
                    };
                    match open_idx {
                        Some(idx) => self.stack[idx] = value,
                        None => {
                            if let Obj::Upvalue(state) = self.heap.get_mut(uv) {
                                *state = UpvalueState::Closed(value);
                            }
                        }
                    }
                }

                Instruction::CLOSE_UPVALUE => {
                    let top = self.stack.len() - 1;
                    self.close_upvalues(top);
                    self.pop();
                }

                _ => panic!("Unknown opcode: 0x{:02x}", opcode),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::BytecodeBuilder;

    #[test]
    fn test_halt() {
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_const() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(42));
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(42)));
    }

    #[test]
    fn test_literals() {
        // Test NULL
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::NULL);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Null));

        // Test TRUE
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::TRUE);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(true)));

        // Test FALSE
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::FALSE);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(false)));
    }

    #[test]
    fn test_add() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(10));
        b.emit_constant(Constant::Int(20));
        b.emit(Instruction::ADD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(30)));
    }

    #[test]
    fn test_add_float() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Float(10.5));
        b.emit_constant(Constant::Float(20.5));
        b.emit(Instruction::ADD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Float(31.0)));
    }

    #[test]
    fn test_string_concat() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Str("Hello, ".into()));
        b.emit_constant(Constant::Str("World!".into()));
        b.emit(Instruction::ADD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(
            result.map(|v| vm.display_value(&v)),
            Some("Hello, World!".to_string())
        );
    }

    #[test]
    fn test_sub() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(50));
        b.emit_constant(Constant::Int(20));
        b.emit(Instruction::SUB);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(30)));
    }

    #[test]
    fn test_mul() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(6));
        b.emit_constant(Constant::Int(7));
        b.emit(Instruction::MUL);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(42)));
    }

    #[test]
    fn test_div() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(100));
        b.emit_constant(Constant::Int(4));
        b.emit(Instruction::DIV);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(25)));
    }

    #[test]
    fn test_mod() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(17));
        b.emit_constant(Constant::Int(5));
        b.emit(Instruction::MOD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(2)));
    }

    #[test]
    fn test_neg() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(42));
        b.emit(Instruction::NEG);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(-42)));
    }

    #[test]
    fn test_comparison() {
        // Test EQ
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(5));
        b.emit_constant(Constant::Int(5));
        b.emit(Instruction::EQ);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(true)));

        // Test NEQ
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(5));
        b.emit_constant(Constant::Int(3));
        b.emit(Instruction::NEQ);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(true)));

        // Test LT
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(3));
        b.emit_constant(Constant::Int(5));
        b.emit(Instruction::LT);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(true)));

        // Test GT
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(5));
        b.emit_constant(Constant::Int(3));
        b.emit(Instruction::GT);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(true)));
    }

    #[test]
    fn test_logical() {
        // Test NOT
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::TRUE);
        b.emit(Instruction::NOT);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(false)));

        // Test AND
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::TRUE);
        b.emit(Instruction::TRUE);
        b.emit(Instruction::AND);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(true)));

        // Test OR
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::FALSE);
        b.emit(Instruction::TRUE);
        b.emit(Instruction::OR);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Bool(true)));
    }

    #[test]
    fn test_bitwise() {
        // Test BITAND
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(0b1100));
        b.emit_constant(Constant::Int(0b1010));
        b.emit(Instruction::BITAND);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Int(0b1000)));

        // Test BITOR
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(0b1100));
        b.emit_constant(Constant::Int(0b1010));
        b.emit(Instruction::BITOR);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec().unwrap(), Some(Value::Int(0b1110)));
    }

    #[test]
    fn test_dup() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(5));
        b.emit(Instruction::DUP);
        b.emit(Instruction::MUL);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(25))); // 5 * 5
    }

    #[test]
    fn test_complex_expression() {
        // Compute: (10 + 20) * 2 - 5 = 55
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(10));
        b.emit_constant(Constant::Int(20));
        b.emit(Instruction::ADD);
        b.emit_constant(Constant::Int(2));
        b.emit(Instruction::MUL);
        b.emit_constant(Constant::Int(5));
        b.emit(Instruction::SUB);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(55)));
    }

    #[test]
    fn test_gc_collects_unreachable() {
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);

        // One list rooted on the stack, three unreachable ones.
        let kept = vm.heap.alloc_list(vec![Value::Int(1)]);
        vm.stack.push(kept.clone());
        for _ in 0..3 {
            vm.heap.alloc_list(vec![Value::Int(0)]);
        }
        assert_eq!(vm.heap.live, 4);

        vm.collect_garbage();
        assert_eq!(vm.heap.live, 1);
        // The survivor is still intact behind its handle.
        assert_eq!(vm.heap.value_len(&kept).unwrap(), 1);
    }

    #[test]
    fn test_gc_traces_object_graph() {
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);

        // outer -> inner: only outer is rooted, but inner must survive too.
        let inner = vm.heap.alloc_str("hi");
        let outer = vm.heap.alloc_list(vec![inner.clone()]);
        vm.stack.push(outer);
        vm.heap.alloc_str("garbage");
        assert_eq!(vm.heap.live, 3);

        vm.collect_garbage();
        assert_eq!(vm.heap.live, 2);
        assert_eq!(vm.display_value(&inner), "hi");
    }

    #[test]
    fn test_gc_reuses_freed_slots() {
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);

        let Value::Obj(dead) = vm.heap.alloc_str("dead") else {
            unreachable!()
        };
        vm.collect_garbage();
        assert_eq!(vm.heap.live, 0);

        // The next allocation should reuse the freed slot, not grow the heap.
        let Value::Obj(reused) = vm.heap.alloc_str("new") else {
            unreachable!()
        };
        assert_eq!(reused, dead);
        assert_eq!(vm.heap.objects.len(), 1);
    }

    #[test]
    fn test_gc_stress_string_concat() {
        // With stress mode on, ADD collects before every string allocation;
        // the interned constants and the result must all survive.
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Str("Hello, ".into()));
        b.emit_constant(Constant::Str("World".into()));
        b.emit(Instruction::ADD);
        b.emit_constant(Constant::Str("!".into()));
        b.emit(Instruction::ADD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        vm.gc_stress = true;
        let result = vm.exec().unwrap();
        assert_eq!(
            result.map(|v| vm.display_value(&v)),
            Some("Hello, World!".to_string())
        );
    }

    #[test]
    fn test_constant_deduplication() {
        let mut b = BytecodeBuilder::new();
        // Use same constant twice - should only be stored once
        b.emit_constant(Constant::Int(42));
        b.emit_constant(Constant::Int(42));
        b.emit(Instruction::ADD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        // Verify deduplication
        assert_eq!(bc.constants.len(), 1);

        let mut vm = AxeVM::new(&bc);
        let result = vm.exec().unwrap();
        assert_eq!(result, Some(Value::Int(84)));
    }
}
