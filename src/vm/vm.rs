use crate::vm::NativeFn;

use super::builtins::builtins;
use super::bytecode::{Bytecode, Constant};
use super::instructions::Instruction;

/// A heap-allocated object. Owned by the VM's `Heap`, never by a `Value`.
#[derive(Debug, PartialEq)]
pub enum Obj {
    Str(String),
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

    /// Dereference a handle to the object it points at.
    fn get(&self, r: ObjRef) -> &Obj {
        &self.objects[r.0]
    }
}

struct Frame {
    ret_ip: usize,
    bp: usize,
}

pub struct AxeVM<'a> {
    bytecode: &'a Bytecode,
    ip: usize,
    bp: usize,
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: Vec<Value>,
    heap: Heap,
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
            globals: globals,
            heap: Heap::new(),
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
                            let result = func(&args, &self.heap);
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::Fn { entry, arity } => {
                            assert_eq!(argc, arity as usize, "wrong arg count");
                            self.frames.push(Frame {
                                ret_ip: self.ip,
                                bp: self.bp,
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
                    self.stack.truncate(self.bp - 1);
                    self.ip = frame.ret_ip;
                    self.bp = frame.bp;
                    self.push(result);
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
