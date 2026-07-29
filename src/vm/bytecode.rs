use fxhash::FxHashMap;

use super::instructions::Instruction;
use crate::Symbol;

/// A compile-time constant baked into the bytecode's constant pool.
///
/// Constants are pure data — they carry no heap handles. String constants
/// are materialized into the VM's heap when the `CONST` opcode loads them,
/// which keeps `Bytecode` self-contained and independent of any VM/heap.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Str(String),
    Fn {
        entry: usize,
        arity: u8,
    },
    /// An interned member name (class/method/property/field). Used as the
    /// operand of the OO opcodes for runtime `Symbol` comparison and lookup —
    /// never loaded onto the stack as a value.
    Sym(Symbol),
}

/// Immutable compiled bytecode ready for execution.
#[derive(Debug, Clone, Default)]
pub struct Bytecode {
    pub code: Vec<u8>,
    pub constants: Vec<Constant>,
    /// Run-length encoded source line table: `(code offset, line)`, sorted by
    /// offset. Each entry marks where the source line changes; `line_at`
    /// resolves any instruction offset back to its line for error reporting.
    pub lines: Vec<(u32, u32)>,
    /// Function name table: `(entry offset, name)`, sorted by entry. Used to
    /// name frames in runtime stack traces.
    pub fn_names: Vec<(usize, String)>,
    /// Source names of interned member symbols, so runtime errors can say
    /// which property/method was involved (the VM has no interner access).
    pub sym_names: FxHashMap<Symbol, String>,
}

impl Bytecode {
    /// Source line for the instruction at `offset` (0 if unknown).
    pub fn line_at(&self, offset: usize) -> u32 {
        match self.lines.binary_search_by_key(&(offset as u32), |e| e.0) {
            Ok(i) => self.lines[i].1,
            Err(0) => 0,
            Err(i) => self.lines[i - 1].1,
        }
    }

    /// Name of the function whose body starts at `entry`, if known.
    pub fn fn_name(&self, entry: usize) -> Option<&str> {
        self.fn_names
            .binary_search_by_key(&entry, |e| e.0)
            .ok()
            .map(|i| self.fn_names[i].1.as_str())
    }

    /// Source name of a member symbol, for error messages.
    pub fn sym_name(&self, sym: Symbol) -> &str {
        self.sym_names.get(&sym).map_or("<unknown>", |s| s.as_str())
    }
}

/// Builder used by the compiler to construct bytecode incrementally.
/// Call `build()` to freeze into a runnable `Bytecode`.
#[derive(Debug, Clone, Default)]
pub struct BytecodeBuilder {
    bytecode: Bytecode,
    /// Source line for bytes emitted from now on; recorded into the RLE
    /// line table on change.
    current_line: u32,
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Freeze the builder into an immutable, runnable `Bytecode`.
    pub fn build(mut self) -> Bytecode {
        self.bytecode.fn_names.sort_by_key(|e| e.0);
        self.bytecode
    }

    /// Set the source line for subsequently emitted bytes. Line 0 (unknown)
    /// is ignored so synthesized nodes don't clobber real line info.
    pub fn set_line(&mut self, line: u32) {
        if line != 0 && line != self.current_line {
            self.current_line = line;
            let offset = self.bytecode.code.len() as u32;
            // Two line changes with no bytes between them: last one wins.
            if let Some(last) = self.bytecode.lines.last_mut()
                && last.0 == offset
            {
                last.1 = line;
            } else {
                self.bytecode.lines.push((offset, line));
            }
        }
    }

    /// Record that a function named `name` has its body entry at `entry`.
    pub fn name_fn(&mut self, entry: usize, name: String) {
        self.bytecode.fn_names.push((entry, name));
    }

    /// Record the source name of a member symbol for error messages.
    pub fn name_sym(&mut self, sym: Symbol, name: String) {
        self.bytecode.sym_names.entry(sym).or_insert(name);
    }

    /// Fallible version of `add_constant` for the compiler: a script with
    /// too many distinct constants gets a compile error, not a panic.
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

    /// Fallible version of `emit_constant` — see `try_add_constant`.
    pub fn try_emit_constant(&mut self, value: Constant) -> Result<(), String> {
        let index = self.try_add_constant(value)?;
        self.emit(Instruction::CONST);
        self.emit(index);
        Ok(())
    }

    /// Add a constant to the pool and return its index.
    /// Returns the existing index if the same value is already present.
    pub fn add_constant(&mut self, value: Constant) -> u8 {
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
    pub fn emit_constant(&mut self, value: Constant) {
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

    /// Emit a backward `LOOP` jump targeting `loop_start`. The VM subtracts the
    /// 2-byte operand from `ip` (which, when read, points just past the operand),
    /// so `operand = (position after operand) - loop_start`.
    pub fn emit_loop(&mut self, loop_start: usize) {
        self.emit(Instruction::LOOP);
        let offset = self.bytecode.code.len() + 2 - loop_start;
        assert!(offset <= u16::MAX as usize, "Loop offset too large");
        let bytes = (offset as u16).to_le_bytes();
        self.emit(bytes[0]);
        self.emit(bytes[1]);
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
