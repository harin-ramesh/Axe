use fxhash::FxHashMap;

use super::instructions::Instruction;
use crate::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Str(String),
    Fn { entry: usize, arity: u8 },
    Sym(Symbol),
}

#[derive(Debug, Clone, Default)]
pub struct Bytecode {
    pub code: Vec<u8>,
    pub constants: Vec<Constant>,
    pub lines: Vec<(u32, u32)>,
    pub fn_names: Vec<(usize, String)>,
    pub sym_names: FxHashMap<Symbol, String>,
}

impl Bytecode {
    pub fn line_at(&self, offset: usize) -> u32 {
        match self.lines.binary_search_by_key(&(offset as u32), |e| e.0) {
            Ok(i) => self.lines[i].1,
            Err(0) => 0,
            Err(i) => self.lines[i - 1].1,
        }
    }

    pub fn fn_name(&self, entry: usize) -> Option<&str> {
        self.fn_names
            .binary_search_by_key(&entry, |e| e.0)
            .ok()
            .map(|i| self.fn_names[i].1.as_str())
    }

    pub fn sym_name(&self, sym: Symbol) -> &str {
        self.sym_names.get(&sym).map_or("<unknown>", |s| s.as_str())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BytecodeBuilder {
    bytecode: Bytecode,
    current_line: u32,
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(mut self) -> Bytecode {
        self.bytecode.fn_names.sort_by_key(|e| e.0);
        self.bytecode
    }

    pub fn set_line(&mut self, line: u32) {
        if line != 0 && line != self.current_line {
            self.current_line = line;
            let offset = self.bytecode.code.len() as u32;
            if let Some(last) = self.bytecode.lines.last_mut()
                && last.0 == offset
            {
                last.1 = line;
            } else {
                self.bytecode.lines.push((offset, line));
            }
        }
    }

    pub fn name_fn(&mut self, entry: usize, name: String) {
        self.bytecode.fn_names.push((entry, name));
    }

    pub fn name_sym(&mut self, sym: Symbol, name: String) {
        self.bytecode.sym_names.entry(sym).or_insert(name);
    }

    pub fn try_add_constant(&mut self, value: Constant) -> Result<u8, String> {
        for (i, existing) in self.bytecode.constants.iter().enumerate() {
            if existing == &value {
                return Ok(i as u8);
            }
        }
        let index = self.bytecode.constants.len();
        if index >= 256 {
            return Err("too many constants in one script (max 256)".to_string());
        }
        self.bytecode.constants.push(value);
        Ok(index as u8)
    }

    pub fn try_emit_constant(&mut self, value: Constant) -> Result<(), String> {
        let index = self.try_add_constant(value)?;
        self.emit(Instruction::CONST);
        self.emit(index);
        Ok(())
    }

    pub fn emit(&mut self, byte: u8) {
        self.bytecode.code.push(byte);
    }

    pub fn here(&self) -> usize {
        self.bytecode.code.len()
    }

    pub fn emit_jump(&mut self, opcode: u8) -> usize {
        self.emit(opcode);
        let offset = self.bytecode.code.len();
        self.emit(0xff);
        self.emit(0xff);
        offset
    }

    pub fn emit_loop(&mut self, loop_start: usize) {
        self.emit(Instruction::LOOP);
        let offset = self.bytecode.code.len() + 2 - loop_start;
        assert!(offset <= u16::MAX as usize, "Loop offset too large");
        let bytes = (offset as u16).to_le_bytes();
        self.emit(bytes[0]);
        self.emit(bytes[1]);
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump = self.bytecode.code.len() - (offset + 2);
        assert!(jump <= u16::MAX as usize, "Jump offset too large");
        let bytes = (jump as u16).to_le_bytes();
        self.bytecode.code[offset] = bytes[0];
        self.bytecode.code[offset + 1] = bytes[1];
    }
}
