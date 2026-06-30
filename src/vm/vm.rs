use std::rc::Rc;

use crate::vm::NativeFn;

use super::bytecode::Bytecode;
use super::instructions::Instruction;
use super::builtins::{builtins};

#[derive(Debug, PartialEq)]
pub enum Obj {
    Str(String),
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Obj(Rc<Obj>),
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

    fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Obj(o) => match o.as_ref() {
                Obj::Str(s) => !s.is_empty(),
            },
            Value::Native(_, _) => true,
            Value::Fn { .. } => true,
        }
    }

    fn is_truthy(&self) -> bool {
        self.as_bool()
    }

    pub fn str(s: impl Into<String>) -> Self {
        Value::Obj(Rc::new(Obj::Str(s.into())))
    }

    pub fn display(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Obj(o) => match o.as_ref() {
                Obj::Str(s) => s.clone(),
            },
            Value::Native(name, _) => format!("<native-fn {}>", name),
            Value::Fn { entry, arity } => format!("<fn @{} /{}>", entry, arity),
        }
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
            globals: globals
        }
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
        self.bytecode.constants[index].clone()
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
                    if !cond.is_truthy() {
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
                        (Value::Obj(ao), Value::Obj(bo)) => match (ao.as_ref(), bo.as_ref()) {
                            (Obj::Str(a), Obj::Str(b)) => Value::str(format!("{}{}", a, b)),
                        },
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
                    self.push(Value::Bool(!a.is_truthy()));
                }

                Instruction::AND => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a.is_truthy() && b.is_truthy()));
                }

                Instruction::OR => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a.is_truthy() || b.is_truthy()));
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
                        self.globals.resize(idx+1, Value::Null)
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
                            let result = func(&args);
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Value::Fn { entry, arity } => {
                            assert_eq!(argc, arity as usize, "wrong arg count");
                            self.frames.push(Frame { ret_ip: self.ip, bp: self.bp });
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
        b.emit_constant(Value::Int(42));
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
        b.emit_constant(Value::Int(10));
        b.emit_constant(Value::Int(20));
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
        b.emit_constant(Value::Float(10.5));
        b.emit_constant(Value::Float(20.5));
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
        b.emit_constant(Value::str("Hello, "));
        b.emit_constant(Value::str("World!"));
        b.emit(Instruction::ADD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        let result = vm.exec();
        assert_eq!(result, Some(Value::str("Hello, World!")));
    }

    #[test]
    fn test_sub() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Value::Int(50));
        b.emit_constant(Value::Int(20));
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
        b.emit_constant(Value::Int(6));
        b.emit_constant(Value::Int(7));
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
        b.emit_constant(Value::Int(100));
        b.emit_constant(Value::Int(4));
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
        b.emit_constant(Value::Int(17));
        b.emit_constant(Value::Int(5));
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
        b.emit_constant(Value::Int(42));
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
        b.emit_constant(Value::Int(5));
        b.emit_constant(Value::Int(5));
        b.emit(Instruction::EQ);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test NEQ
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Value::Int(5));
        b.emit_constant(Value::Int(3));
        b.emit(Instruction::NEQ);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test LT
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Value::Int(3));
        b.emit_constant(Value::Int(5));
        b.emit(Instruction::LT);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test GT
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Value::Int(5));
        b.emit_constant(Value::Int(3));
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
        b.emit_constant(Value::Int(0b1100));
        b.emit_constant(Value::Int(0b1010));
        b.emit(Instruction::BITAND);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Int(0b1000)));

        // Test BITOR
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Value::Int(0b1100));
        b.emit_constant(Value::Int(0b1010));
        b.emit(Instruction::BITOR);
        b.emit(Instruction::HALT);
        let bc = b.build();
        let mut vm = AxeVM::new(&bc);
        assert_eq!(vm.exec(), Some(Value::Int(0b1110)));
    }

    #[test]
    fn test_dup() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Value::Int(5));
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
        b.emit_constant(Value::Int(10));
        b.emit_constant(Value::Int(20));
        b.emit(Instruction::ADD);
        b.emit_constant(Value::Int(2));
        b.emit(Instruction::MUL);
        b.emit_constant(Value::Int(5));
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
        b.emit_constant(Value::Int(42));
        b.emit_constant(Value::Int(42));
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
