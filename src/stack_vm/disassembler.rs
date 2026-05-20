use std::fmt::Write;

use super::instructions::Instruction;
use super::vm::{Chunk, Obj, Value};

/// Disassemble a chunk into a human-readable string.
///
/// Output layout (offset, raw bytes, opcode, operand):
///   == <name> ==
///   0000  01 00     CONST          Int(10)
///   0002  01 01     CONST          Int(20)
///   0004  10        ADD
///   0005  00        HALT
pub fn disassemble_chunk(chunk: &Chunk, name: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "== {} ==", name);

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset, &mut out);
    }
    out
}

/// Width reserved for the raw-bytes column. Largest instruction is 3 bytes
/// (e.g. JUMP) which renders as "XX XX XX" = 8 chars.
const BYTES_COL_WIDTH: usize = 8;

/// Disassemble a single instruction, append it to `out`, and return the
/// offset of the next instruction.
pub fn disassemble_instruction(chunk: &Chunk, offset: usize, out: &mut String) -> usize {
    let opcode = chunk.code[offset];
    match opcode {
        Instruction::HALT => simple(out, "HALT", chunk, offset),

        Instruction::CONST => constant(out, "CONST", chunk, offset),
        Instruction::POP => simple(out, "POP", chunk, offset),
        Instruction::DUP => simple(out, "DUP", chunk, offset),

        Instruction::NULL => simple(out, "NULL", chunk, offset),
        Instruction::TRUE => simple(out, "TRUE", chunk, offset),
        Instruction::FALSE => simple(out, "FALSE", chunk, offset),

        Instruction::ADD => simple(out, "ADD", chunk, offset),
        Instruction::SUB => simple(out, "SUB", chunk, offset),
        Instruction::MUL => simple(out, "MUL", chunk, offset),
        Instruction::DIV => simple(out, "DIV", chunk, offset),
        Instruction::MOD => simple(out, "MOD", chunk, offset),
        Instruction::NEG => simple(out, "NEG", chunk, offset),

        Instruction::EQ => simple(out, "EQ", chunk, offset),
        Instruction::NEQ => simple(out, "NEQ", chunk, offset),
        Instruction::LT => simple(out, "LT", chunk, offset),
        Instruction::LTE => simple(out, "LTE", chunk, offset),
        Instruction::GT => simple(out, "GT", chunk, offset),
        Instruction::GTE => simple(out, "GTE", chunk, offset),

        Instruction::NOT => simple(out, "NOT", chunk, offset),
        Instruction::AND => simple(out, "AND", chunk, offset),
        Instruction::OR => simple(out, "OR", chunk, offset),

        Instruction::BITAND => simple(out, "BITAND", chunk, offset),
        Instruction::BITOR => simple(out, "BITOR", chunk, offset),
        Instruction::BITINV => simple(out, "BITINV", chunk, offset),

        Instruction::JUMP => jump(out, "JUMP", chunk, offset),
        Instruction::JUMP_IF_FALSE => jump(out, "JUMP_IF_FALSE", chunk, offset),

        unknown => {
            write_prefix(out, chunk, offset, 1);
            let _ = writeln!(out, "<unknown 0x{:02x}>", unknown);
            offset + 1
        }
    }
}

/// Write `OFFSET  BB BB BB  ` prefix (offset + raw bytes column).
fn write_prefix(out: &mut String, chunk: &Chunk, offset: usize, size: usize) {
    let _ = write!(out, "{:04}  ", offset);
    let mut bytes = String::new();
    for i in 0..size {
        if i > 0 {
            bytes.push(' ');
        }
        let _ = write!(bytes, "{:02x}", chunk.code[offset + i]);
    }
    let _ = write!(out, "{:<width$}  ", bytes, width = BYTES_COL_WIDTH);
}

fn simple(out: &mut String, name: &str, chunk: &Chunk, offset: usize) -> usize {
    write_prefix(out, chunk, offset, 1);
    let _ = writeln!(out, "{}", name);
    offset + 1
}

fn constant(out: &mut String, name: &str, chunk: &Chunk, offset: usize) -> usize {
    write_prefix(out, chunk, offset, 2);
    let idx = chunk.code[offset + 1];
    let value = chunk
        .constants
        .get(idx as usize)
        .map(format_value)
        .unwrap_or_else(|| "<out of range>".to_string());
    let _ = writeln!(out, "{:<14} {}", name, value);
    offset + 2
}

fn jump(out: &mut String, name: &str, chunk: &Chunk, offset: usize) -> usize {
    write_prefix(out, chunk, offset, 3);
    let lo = chunk.code[offset + 1];
    let hi = chunk.code[offset + 2];
    let delta = u16::from_le_bytes([lo, hi]) as usize;
    // The VM reads the 2-byte operand and then adds the delta, so the target
    // is the byte after the operand plus the delta.
    let target = offset + 3 + delta;
    let _ = writeln!(out, "{:<14} -> {:04}", name, target);
    offset + 3
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Null => "Null".to_string(),
        Value::Bool(b) => format!("Bool({})", b),
        Value::Int(n) => format!("Int({})", n),
        Value::Float(n) => format!("Float({})", n),
        Value::Obj(o) => match o.as_ref() {
            Obj::Str(s) => format!("Str({:?})", s),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_vm::Instruction;

    #[test]
    fn simple_opcodes() {
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::TRUE);
        chunk.emit(Instruction::NOT);
        chunk.emit(Instruction::HALT);

        let dis = disassemble_chunk(&chunk, "test");
        assert!(dis.contains("== test =="));
        // offset + raw byte + opcode name
        assert!(dis.contains("0000  05        TRUE"));
        assert!(dis.contains("0001  30        NOT"));
        assert!(dis.contains("0002  00        HALT"));
    }

    #[test]
    fn constant_opcode_inlines_value() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Int(42));
        chunk.emit_constant(Value::str("hello"));
        chunk.emit(Instruction::ADD);
        chunk.emit(Instruction::HALT);

        let dis = disassemble_chunk(&chunk, "consts");
        // No separate constants section anymore.
        assert!(!dis.contains("Constants:"));
        // Raw bytes column should show opcode (01) + index byte.
        assert!(dis.contains("0000  01 00     CONST          Int(42)"));
        assert!(dis.contains("0002  01 01     CONST          Str(\"hello\")"));
    }

    #[test]
    fn jump_targets_are_resolved() {
        // if (true) {} else {} -> emits TRUE, JUMP_IF_FALSE, JUMP, HALT
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::TRUE);
        let jif = chunk.emit_jump(Instruction::JUMP_IF_FALSE);
        // then branch is empty
        let jmp = chunk.emit_jump(Instruction::JUMP);
        chunk.patch_jump(jif);
        // else branch is empty
        chunk.patch_jump(jmp);
        chunk.emit(Instruction::HALT);

        let dis = disassemble_chunk(&chunk, "jumps");
        // JUMP_IF_FALSE at 0001 skips the unconditional JUMP (3 bytes) to 0007.
        // Layout: 0000 TRUE, 0001 JUMP_IF_FALSE (3 bytes), 0004 JUMP (3 bytes), 0007 HALT.
        // Both jumps skip the empty branch and land at 0007.
        assert!(dis.contains("0001  51 03 00  JUMP_IF_FALSE  -> 0007"));
        assert!(dis.contains("0004  50 00 00  JUMP           -> 0007"));
    }

    #[test]
    fn unknown_opcode_does_not_panic() {
        let mut chunk = Chunk::new();
        chunk.emit(0xEE);
        let dis = disassemble_chunk(&chunk, "bad");
        assert!(dis.contains("<unknown 0xee>"));
    }
}
