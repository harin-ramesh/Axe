use super::instructions::Instruction;
use super::vm::Value;

/// Immutable compiled bytecode ready for execution.
#[derive(Debug, Clone, Default)]
pub struct Bytecode {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

/// Builder used by the compiler to construct bytecode incrementally.
/// Call `build()` to freeze into a runnable `Bytecode`.
#[derive(Debug, Clone, Default)]
pub struct BytecodeBuilder {
    bytecode: Bytecode,
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Freeze the builder into an immutable, runnable `Bytecode`.
    pub fn build(self) -> Bytecode {
        self.bytecode
    }

    /// Add a constant to the pool and return its index.
    /// Returns the existing index if the same value is already present.
    pub fn add_constant(&mut self, value: Value) -> u8 {
        for (i, existing) in self.bytecode.constants.iter().enumerate() {
            if existing == &value {
                return i as u8;
            }
        }
        let index = self.bytecode.constants.len();
        assert!(index < 256, "Too many constants in bytecode");
        self.bytecode.constants.push(value);
        index as u8
    }

    /// Emit a single byte.
    pub fn emit(&mut self, byte: u8) {
        self.bytecode.code.push(byte);
    }

    /// Emit a constant load instruction.
    pub fn emit_constant(&mut self, value: Value) {
        let index = self.add_constant(value);
        self.emit(Instruction::CONST);
        self.emit(index);
    }

    pub fn here(&self) -> usize {
        self.bytecode.code.len()
    }

    /// Emit a jump opcode followed by a 2-byte placeholder offset.
    /// Returns the index of the first placeholder byte so it can be patched later.
    pub fn emit_jump(&mut self, opcode: u8) -> usize {
        self.emit(opcode);
        let offset = self.bytecode.code.len();
        self.emit(0xff);
        self.emit(0xff);
        offset
    }

    /// Patch a previously emitted jump so it targets the current end of the bytecode.
    pub fn patch_jump(&mut self, offset: usize) {
        let jump = self.bytecode.code.len() - (offset + 2);
        assert!(jump <= u16::MAX as usize, "Jump offset too large");
        let bytes = (jump as u16).to_le_bytes();
        self.bytecode.code[offset] = bytes[0];
        self.bytecode.code[offset + 1] = bytes[1];
    }
}
