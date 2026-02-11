use crate::interpreter::instructions::Instruction;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl Value {
    fn as_number(&self) -> f64 {
        match self {
            Value::Int(n) => *n as f64,
            Value::Float(n) => *n,
            _ => panic!("Expected number, got {:?}", self),
        }
    }

    fn as_int(&self) -> i64 {
        match self {
            Value::Int(n) => *n,
            Value::Float(n) => *n as i64,
            _ => panic!("Expected integer, got {:?}", self),
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
        }
    }

    fn is_truthy(&self) -> bool {
        self.as_bool()
    }
}

/// Compiled bytecode chunk with constant pool
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, value: Value) -> u8 {
        // Check if constant already exists
        for (i, existing) in self.constants.iter().enumerate() {
            if existing == &value {
                return i as u8;
            }
        }
        let index = self.constants.len();
        assert!(index < 256, "Too many constants in chunk");
        self.constants.push(value);
        index as u8
    }

    /// Emit a single byte
    pub fn emit(&mut self, byte: u8) {
        self.code.push(byte);
    }

    /// Emit a constant load instruction
    pub fn emit_constant(&mut self, value: Value) {
        let index = self.add_constant(value);
        self.emit(Instruction::CONST);
        self.emit(index);
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AxeVM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl<'a> AxeVM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        AxeVM {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(256),
        }
    }

    pub fn exec(&mut self) -> Option<Value> {
        self.ip = 0;
        self.stack.clear();
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
        let value = self.chunk.code[self.ip];
        self.ip += 1;
        value
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_u8() as usize;
        self.chunk.constants[index].clone()
    }

    fn eval(&mut self) {
        loop {
            let opcode = self.read_u8();
            match opcode {
                Instruction::HALT => break,

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
                        (Value::Str(a), Value::Str(b)) => Value::Str(format!("{}{}", a, b)),
                        _ => Value::Float(a.as_number() + b.as_number()),
                    };
                    self.push(result);
                }

                Instruction::SUB => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                        _ => Value::Float(a.as_number() - b.as_number()),
                    };
                    self.push(result);
                }

                Instruction::MUL => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                        _ => Value::Float(a.as_number() * b.as_number()),
                    };
                    self.push(result);
                }

                Instruction::DIV => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
                        _ => Value::Float(a.as_number() / b.as_number()),
                    };
                    self.push(result);
                }

                Instruction::MOD => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = match (&a, &b) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
                        _ => Value::Float(a.as_number() % b.as_number()),
                    };
                    self.push(result);
                }

                Instruction::NEG => {
                    let a = self.pop();
                    let result = match a {
                        Value::Int(n) => Value::Int(-n),
                        _ => Value::Float(-a.as_number()),
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
                    let b = self.pop().as_number();
                    let a = self.pop().as_number();
                    self.push(Value::Bool(a < b));
                }

                Instruction::LTE => {
                    let b = self.pop().as_number();
                    let a = self.pop().as_number();
                    self.push(Value::Bool(a <= b));
                }

                Instruction::GT => {
                    let b = self.pop().as_number();
                    let a = self.pop().as_number();
                    self.push(Value::Bool(a > b));
                }

                Instruction::GTE => {
                    let b = self.pop().as_number();
                    let a = self.pop().as_number();
                    self.push(Value::Bool(a >= b));
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

                _ => panic!("Unknown opcode: 0x{:02x}", opcode),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_halt() {
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert!(result.is_none());
    }

    #[test]
    fn test_const() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(42));
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(42)));
    }

    #[test]
    fn test_literals() {
        // Test NULL
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::NULL);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Null));

        // Test TRUE
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::TRUE);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test FALSE
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::FALSE);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(false)));
    }

    #[test]
    fn test_add() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(10));
        chunk.emit_constant(Value::Int(20));
        chunk.emit(Instruction::ADD);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(30)));
    }

    #[test]
    fn test_add_float() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Float(10.5));
        chunk.emit_constant(Value::Float(20.5));
        chunk.emit(Instruction::ADD);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Float(31.0)));
    }

    #[test]
    fn test_string_concat() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Str("Hello, ".to_string()));
        chunk.emit_constant(Value::Str("World!".to_string()));
        chunk.emit(Instruction::ADD);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Str("Hello, World!".to_string())));
    }

    #[test]
    fn test_sub() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(50));
        chunk.emit_constant(Value::Int(20));
        chunk.emit(Instruction::SUB);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(30)));
    }

    #[test]
    fn test_mul() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(6));
        chunk.emit_constant(Value::Int(7));
        chunk.emit(Instruction::MUL);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(42)));
    }

    #[test]
    fn test_div() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(100));
        chunk.emit_constant(Value::Int(4));
        chunk.emit(Instruction::DIV);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(25)));
    }

    #[test]
    fn test_mod() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(17));
        chunk.emit_constant(Value::Int(5));
        chunk.emit(Instruction::MOD);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(2)));
    }

    #[test]
    fn test_neg() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(42));
        chunk.emit(Instruction::NEG);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(-42)));
    }

    #[test]
    fn test_comparison() {
        // Test EQ
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(5));
        chunk.emit_constant(Value::Int(5));
        chunk.emit(Instruction::EQ);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test NEQ
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(5));
        chunk.emit_constant(Value::Int(3));
        chunk.emit(Instruction::NEQ);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test LT
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(3));
        chunk.emit_constant(Value::Int(5));
        chunk.emit(Instruction::LT);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test GT
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(5));
        chunk.emit_constant(Value::Int(3));
        chunk.emit(Instruction::GT);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));
    }

    #[test]
    fn test_logical() {
        // Test NOT
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::TRUE);
        chunk.emit(Instruction::NOT);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(false)));

        // Test AND
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::TRUE);
        chunk.emit(Instruction::TRUE);
        chunk.emit(Instruction::AND);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));

        // Test OR
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::FALSE);
        chunk.emit(Instruction::TRUE);
        chunk.emit(Instruction::OR);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Bool(true)));
    }

    #[test]
    fn test_bitwise() {
        // Test BITAND
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(0b1100));
        chunk.emit_constant(Value::Int(0b1010));
        chunk.emit(Instruction::BITAND);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Int(0b1000)));

        // Test BITOR
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(0b1100));
        chunk.emit_constant(Value::Int(0b1010));
        chunk.emit(Instruction::BITOR);
        chunk.emit(Instruction::HALT);
        let mut vm = AxeVM::new(&chunk);
        assert_eq!(vm.exec(), Some(Value::Int(0b1110)));
    }

    #[test]
    fn test_dup() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(5));
        chunk.emit(Instruction::DUP);
        chunk.emit(Instruction::MUL);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(25))); // 5 * 5
    }

    #[test]
    fn test_complex_expression() {
        // Compute: (10 + 20) * 2 - 5 = 55
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(10));
        chunk.emit_constant(Value::Int(20));
        chunk.emit(Instruction::ADD);
        chunk.emit_constant(Value::Int(2));
        chunk.emit(Instruction::MUL);
        chunk.emit_constant(Value::Int(5));
        chunk.emit(Instruction::SUB);
        chunk.emit(Instruction::HALT);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(55)));
    }

    #[test]
    fn test_constant_deduplication() {
        let mut chunk = Chunk::new();
        // Use same constant twice - should only be stored once
        chunk.emit_constant(Value::Int(42));
        chunk.emit_constant(Value::Int(42));
        chunk.emit(Instruction::ADD);
        chunk.emit(Instruction::HALT);

        // Verify deduplication
        assert_eq!(chunk.constants.len(), 1);

        let mut vm = AxeVM::new(&chunk);
        let result = vm.exec();
        assert_eq!(result, Some(Value::Int(84)));
    }
}
