use fxhash::FxHashMap;

use crate::Symbol;
use crate::vm::NativeFn;

use super::builtins::builtins;
use super::bytecode::{Bytecode, Constant};
use super::instructions::Instruction;

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
    fn as_int(&self) -> i64 {
        match self {
            Value::Int(n) => *n,
            _ => panic!("Expected Int, got {:?}", self),
        }
    }

    #[allow(dead_code)]
    fn as_float(&self) -> f64 {
        match self {
            Value::Float(n) => *n,
            _ => panic!("Expected Float, got {:?}", self),
        }
    }

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

/// The VM-owned object store. All heap objects live here; `Value`s only
/// reference them through `ObjRef` handles.
pub struct Heap {
    objects: Vec<Obj>,
}

impl Heap {
    fn new() -> Self {
        Heap {
            objects: Vec::new(),
        }
    }

    /// Allocate an object and return a handle to it.
    fn alloc(&mut self, obj: Obj) -> ObjRef {
        let index = self.objects.len();
        self.objects.push(obj);
        ObjRef(index)
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
    pub fn value_len(&self, value: &Value) -> i64 {
        match value {
            Value::Obj(o) => match self.get(*o) {
                Obj::List(items) => items.len() as i64,
                Obj::Str(s) => s.chars().count() as i64,
                _ => panic!("value has no length: {:?}", value),
            },
            _ => panic!("value has no length: {:?}", value),
        }
    }

    /// Dereference a handle to the object it points at.
    fn get(&self, r: ObjRef) -> &Obj {
        &self.objects[r.0]
    }

    /// Mutably dereference a handle to the object it points at.
    fn get_mut(&mut self, r: ObjRef) -> &mut Obj {
        &mut self.objects[r.0]
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
        }
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

    pub fn exec(&mut self) -> Option<Value> {
        self.ip = 0;
        self.bp = 0;
        self.stack.clear();
        self.frames.clear();
        self.open_upvalues.clear();
        self.eval();
        self.stack.pop()
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
        match self.bytecode.constants[index].clone() {
            Constant::Int(n) => Value::Int(n),
            Constant::Float(n) => Value::Float(n),
            Constant::Fn { entry, arity } => Value::Fn { entry, arity },
            Constant::Str(s) => self.heap.alloc_str(s),
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

    fn eval(&mut self) {
        loop {
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
                        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                        (Value::Obj(ao), Value::Obj(bo)) => {
                            let s = match (self.heap.get(*ao), self.heap.get(*bo)) {
                                (Obj::Str(a), Obj::Str(b)) => format!("{}{}", a, b),
                                _ => panic!("type error: + on non-string objects"),
                            };
                            self.heap.alloc_str(s)
                        }
                        _ => panic!("type error: {:?} + {:?}", a, b),
                    };
                    self.push(result);
                }

                Instruction::SUB => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        _ => panic!("type error: {:?} - {:?}", a, b),
                    };
                    self.push(result);
                }

                Instruction::MUL => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                        _ => panic!("type error: {:?} * {:?}", a, b),
                    };
                    self.push(result);
                }

                Instruction::DIV => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                        _ => panic!("type error: {:?} / {:?}", a, b),
                    };
                    self.push(result);
                }

                Instruction::MOD => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                        _ => panic!("type error: {:?} % {:?}", a, b),
                    };
                    self.push(result);
                }

                Instruction::NEG => {
                    let a = self.pop();
                    let result = match a {
                        Value::Int(n) => Value::Int(-n),
                        Value::Float(n) => Value::Float(-n),
                        _ => panic!("type error: -{:?}", a),
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
                        _ => panic!("type error: {:?} < {:?}", a, b),
                    };
                    self.push(Value::Bool(result));
                }

                Instruction::LTE => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => a <= b,
                        (Value::Float(a), Value::Float(b)) => a <= b,
                        _ => panic!("type error: {:?} <= {:?}", a, b),
                    };
                    self.push(Value::Bool(result));
                }

                Instruction::GT => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => a > b,
                        (Value::Float(a), Value::Float(b)) => a > b,
                        _ => panic!("type error: {:?} > {:?}", a, b),
                    };
                    self.push(Value::Bool(result));
                }

                Instruction::GTE => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => a >= b,
                        (Value::Float(a), Value::Float(b)) => a >= b,
                        _ => panic!("type error: {:?} >= {:?}", a, b),
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
                    let b = self.pop().as_int();
                    let a = self.pop().as_int();
                    self.push(Value::Int(a & b));
                }

                Instruction::BITOR => {
                    let b = self.pop().as_int();
                    let a = self.pop().as_int();
                    self.push(Value::Int(a | b));
                }

                Instruction::BITINV => {
                    let a = self.pop().as_int();
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
                        Value::Native(_, func) => {
                            let args: Vec<Value> = self.stack[callee_idx + 1..].to_vec();
                            let result = func(&args, &mut self.heap);
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::Fn { entry, arity } => {
                            assert_eq!(argc, arity as usize, "wrong arg count");
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: NO_CLOSURE,
                            });
                            self.bp = callee_idx + 1;
                            self.ip = entry;
                        }
                        Value::Obj(closure_ref) => {
                            let (entry, arity) = match self.heap.get(closure_ref) {
                                Obj::Closure { entry, arity, .. } => (*entry, *arity),
                                other => panic!("not callable: {:?}", other),
                            };
                            assert_eq!(argc, arity as usize, "wrong arg count");
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: closure_ref.0,
                            });
                            self.bp = callee_idx + 1;
                            self.ip = entry;
                        }
                        other => panic!("not callable: {:?}", other),
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
                    let class = self.heap.alloc_class(name);
                    self.push(class);
                }

                Instruction::INHERIT => {
                    let superclass = self.pop();
                    let class = self.peek().clone();
                    let (Value::Obj(class_ref), Value::Obj(super_ref)) = (class, superclass) else {
                        panic!("can only inherit between classes");
                    };
                    assert!(
                        matches!(self.heap.get(super_ref), Obj::Class { .. }),
                        "superclass is not a class"
                    );
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
                    let Value::Obj(obj_ref) = self.pop() else {
                        panic!("cannot access property on non-object");
                    };
                    let (field, class) = match self.heap.get(obj_ref) {
                        Obj::Instance { fields, class } => (fields.get(&name).cloned(), *class),
                        _ => panic!("cannot access property on non-instance"),
                    };
                    let value = field
                        .or_else(|| self.heap.find_method(class, name))
                        .or_else(|| self.heap.find_static(class, name))
                        .unwrap_or_else(|| panic!("property not found"));
                    self.push(value);
                }

                Instruction::SET_PROPERTY => {
                    let name = self.read_sym();
                    let value = self.pop();
                    let Value::Obj(obj_ref) = self.pop() else {
                        panic!("cannot set property on non-object");
                    };
                    if let Obj::Instance { fields, .. } = self.heap.get_mut(obj_ref) {
                        fields.insert(name, value.clone());
                    } else {
                        panic!("cannot set property on non-instance");
                    }
                    self.push(value);
                }

                Instruction::GET_STATIC => {
                    let name = self.read_sym();
                    let Value::Obj(class_ref) = self.pop() else {
                        panic!("cannot access static member on non-class");
                    };
                    let value = self
                        .heap
                        .find_static(class_ref, name)
                        .or_else(|| self.heap.find_method(class_ref, name))
                        .unwrap_or_else(|| panic!("static member not found"));
                    self.push(value);
                }

                Instruction::NEW => {
                    let init_name = self.read_sym();
                    let argc = self.read_u8() as usize;
                    let class_idx = self.stack.len() - argc - 1;
                    let Value::Obj(class_ref) = self.stack[class_idx] else {
                        panic!("can only 'new' a class");
                    };
                    assert!(
                        matches!(self.heap.get(class_ref), Obj::Class { .. }),
                        "can only 'new' a class"
                    );
                    let instance = self.heap.alloc_instance(class_ref);

                    match self.heap.find_method(class_ref, init_name) {
                        Some(Value::Fn { entry, arity }) => {
                            // init receives (self, args...): arity counts self.
                            assert_eq!(arity as usize, argc + 1, "wrong arg count to init");
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
                            });
                            self.bp = class_idx + 1;
                            self.ip = entry;
                        }
                        None => {
                            // No constructor: discard args, yield the bare instance.
                            self.stack.truncate(class_idx);
                            self.push(instance);
                        }
                        Some(other) => panic!("init is not a function: {:?}", other),
                    }
                }

                Instruction::INVOKE => {
                    let name = self.read_sym();
                    let argc = self.read_u8() as usize;
                    let recv_idx = self.stack.len() - argc - 1;
                    let Value::Obj(obj_ref) = self.stack[recv_idx] else {
                        panic!("cannot call method on non-object");
                    };
                    let class = match self.heap.get(obj_ref) {
                        Obj::Instance { class, .. } => *class,
                        _ => panic!("cannot call method on this type"),
                    };
                    match self.heap.find_method(class, name) {
                        Some(Value::Fn { entry, arity }) => {
                            // method receives (self, args...): arity counts self.
                            assert_eq!(arity as usize, argc + 1, "wrong arg count to method");
                            // Insert the callee below the receiver so the receiver
                            // becomes slot 0 (self) of the new frame.
                            self.stack.insert(recv_idx, Value::Fn { entry, arity });
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: NO_CLOSURE,
                            });
                            self.bp = recv_idx + 1;
                            self.ip = entry;
                        }
                        _ => panic!("method not found"),
                    }
                }

                Instruction::STATIC_INVOKE => {
                    let name = self.read_sym();
                    let argc = self.read_u8() as usize;
                    let class_idx = self.stack.len() - argc - 1;
                    let Value::Obj(class_ref) = self.stack[class_idx] else {
                        panic!("cannot call static method on non-class");
                    };
                    let method = self
                        .heap
                        .find_method(class_ref, name)
                        .or_else(|| self.heap.find_static(class_ref, name));
                    match method {
                        Some(Value::Fn { entry, arity }) => {
                            assert_eq!(arity as usize, argc, "wrong arg count to static method");
                            // Replace the class with the callee; args are slots 0..
                            self.stack[class_idx] = Value::Fn { entry, arity };
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
                                return_override: None,
                                closure: NO_CLOSURE,
                            });
                            self.bp = class_idx + 1;
                            self.ip = entry;
                        }
                        _ => panic!("static method not found"),
                    }
                }

                Instruction::BUILD_LIST => {
                    let count = self.read_u8() as usize;
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
                        other => panic!("list index must be an integer, got {:?}", other),
                    };
                    let Value::Obj(obj_ref) = list else {
                        panic!("cannot index non-list");
                    };
                    let element = match self.heap.get(obj_ref) {
                        Obj::List(items) => {
                            let len = items.len() as i64;
                            let resolved = if idx < 0 { idx + len } else { idx };
                            if resolved < 0 || resolved >= len {
                                panic!("list index out of bounds: {}", idx);
                            }
                            items[resolved as usize].clone()
                        }
                        _ => panic!("cannot index non-list"),
                    };
                    self.push(element);
                }

                Instruction::LEN => {
                    let value = self.pop();
                    let len = self.heap.value_len(&value);
                    self.push(Value::Int(len));
                }

                Instruction::CLOSURE => {
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
        let result = vm.exec();
        assert!(result.is_none());
    }

    #[test]
    fn test_const() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(42));
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec();
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
        assert_eq!(vm.exec(), Some(Value::Null));

        // Test TRUE
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::TRUE);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test FALSE
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::FALSE);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(false)));
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
        let result = vm.exec();
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
        let result = vm.exec();
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
        let result = vm.exec();
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
        let result = vm.exec();
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
        let result = vm.exec();
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
        let result = vm.exec();
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
        let result = vm.exec();
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
        let result = vm.exec();
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
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test NEQ
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(5));
        b.emit_constant(Constant::Int(3));
        b.emit(Instruction::NEQ);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test LT
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(3));
        b.emit_constant(Constant::Int(5));
        b.emit(Instruction::LT);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test GT
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(5));
        b.emit_constant(Constant::Int(3));
        b.emit(Instruction::GT);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));
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
        assert_eq!(vm.exec(), Some(Value::Bool(false)));

        // Test AND
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::TRUE);
        b.emit(Instruction::TRUE);
        b.emit(Instruction::AND);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test OR
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::FALSE);
        b.emit(Instruction::TRUE);
        b.emit(Instruction::OR);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));
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
        assert_eq!(vm.exec(), Some(Value::Int(0b1000)));

        // Test BITOR
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(0b1100));
        b.emit_constant(Constant::Int(0b1010));
        b.emit(Instruction::BITOR);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Int(0b1110)));
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
        let result = vm.exec();
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
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(55)));
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
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(84)));
    }
}
