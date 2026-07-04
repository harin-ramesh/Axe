use std::fmt::Write;

use super::bytecode::{Bytecode, Constant};
use super::instructions::Instruction;

/// Disassemble bytecode into a human-readable string.
///
/// Output layout (offset, raw bytes, opcode, operand):
///   0000  01 00     CONST          Int(10)
///   0002  01 01     CONST          Int(20)
///   0004  10        ADD
///   0005  00        HALT
pub fn disassemble(bytecode: &Bytecode) -> String {
    let mut out = String::new();

    let mut offset = 0;
    while offset < bytecode.code.len() {
        offset = disassemble_instruction(bytecode, offset, &mut out);
    }
    out
}

/// Width reserved for the raw-bytes column. Largest instruction is 3 bytes
/// (e.g. JUMP) which renders as "XX XX XX" = 8 chars.
const BYTES_COL_WIDTH: usize = 8;

/// Disassemble a single instruction, append it to `out`, and return the
/// offset of the next instruction.
pub fn disassemble_instruction(bytecode: &Bytecode, offset: usize, out: &mut String) -> usize {
    let opcode = bytecode.code[offset];
    match opcode {
        Instruction::HALT => simple(out, "HALT", bytecode, offset),

        Instruction::CONST => constant(out, "CONST", bytecode, offset),
        Instruction::POP => simple(out, "POP", bytecode, offset),
        Instruction::DUP => simple(out, "DUP", bytecode, offset),

        Instruction::NULL => simple(out, "NULL", bytecode, offset),
        Instruction::TRUE => simple(out, "TRUE", bytecode, offset),
        Instruction::FALSE => simple(out, "FALSE", bytecode, offset),

        Instruction::ADD => simple(out, "ADD", bytecode, offset),
        Instruction::SUB => simple(out, "SUB", bytecode, offset),
        Instruction::MUL => simple(out, "MUL", bytecode, offset),
        Instruction::DIV => simple(out, "DIV", bytecode, offset),
        Instruction::MOD => simple(out, "MOD", bytecode, offset),
        Instruction::NEG => simple(out, "NEG", bytecode, offset),

        Instruction::EQ => simple(out, "EQ", bytecode, offset),
        Instruction::NEQ => simple(out, "NEQ", bytecode, offset),
        Instruction::LT => simple(out, "LT", bytecode, offset),
        Instruction::LTE => simple(out, "LTE", bytecode, offset),
        Instruction::GT => simple(out, "GT", bytecode, offset),
        Instruction::GTE => simple(out, "GTE", bytecode, offset),

        Instruction::NOT => simple(out, "NOT", bytecode, offset),
        Instruction::AND => simple(out, "AND", bytecode, offset),
        Instruction::OR => simple(out, "OR", bytecode, offset),

        Instruction::BITAND => simple(out, "BITAND", bytecode, offset),
        Instruction::BITOR => simple(out, "BITOR", bytecode, offset),
        Instruction::BITINV => simple(out, "BITINV", bytecode, offset),

        Instruction::JUMP => jump(out, "JUMP", bytecode, offset),
        Instruction::JUMP_IF_FALSE => jump(out, "JUMP_IF_FALSE", bytecode, offset),
        Instruction::LOOP => loop_jump(out, "LOOP", bytecode, offset),

        Instruction::BUILD_LIST => byte_operand(out, "BUILD_LIST", bytecode, offset),
        Instruction::GET_INDEX => simple(out, "GET_INDEX", bytecode, offset),
        Instruction::LEN => simple(out, "LEN", bytecode, offset),

        Instruction::CLOSURE => closure(out, bytecode, offset),
        Instruction::GET_UPVALUE => byte_operand(out, "GET_UPVALUE", bytecode, offset),
        Instruction::SET_UPVALUE => byte_operand(out, "SET_UPVALUE", bytecode, offset),
        Instruction::CLOSE_UPVALUE => simple(out, "CLOSE_UPVALUE", bytecode, offset),

        Instruction::DEFINE_GLOBAL => byte_operand(out, "DEFINE_GLOBAL", bytecode, offset),
        Instruction::SET_GLOBAL => byte_operand(out, "SET_GLOBAL", bytecode, offset),
        Instruction::GET_GLOBAL => byte_operand(out, "GET_GLOBAL", bytecode, offset),

        Instruction::DEFINE_LOCAL => byte_operand(out, "DEFINE_LOCAL", bytecode, offset),
        Instruction::SET_LOCAL => byte_operand(out, "SET_LOCAL", bytecode, offset),
        Instruction::GET_LOCAL => byte_operand(out, "GET_LOCAL", bytecode, offset),

        Instruction::CALL => byte_operand(out, "CALL", bytecode, offset),
        Instruction::RETURN => simple(out, "RETURN", bytecode, offset),

        Instruction::CLASS => constant(out, "CLASS", bytecode, offset),
        Instruction::INHERIT => simple(out, "INHERIT", bytecode, offset),
        Instruction::METHOD => constant(out, "METHOD", bytecode, offset),
        Instruction::STATIC_FIELD => constant(out, "STATIC_FIELD", bytecode, offset),
        Instruction::GET_PROPERTY => constant(out, "GET_PROPERTY", bytecode, offset),
        Instruction::SET_PROPERTY => constant(out, "SET_PROPERTY", bytecode, offset),
        Instruction::GET_STATIC => constant(out, "GET_STATIC", bytecode, offset),
        Instruction::NEW => invoke(out, "NEW", bytecode, offset),
        Instruction::INVOKE => invoke(out, "INVOKE", bytecode, offset),
        Instruction::STATIC_INVOKE => invoke(out, "STATIC_INVOKE", bytecode, offset),

        unknown => {
            write_prefix(out, bytecode, offset, 1);
            let _ = writeln!(out, "<unknown 0x{:02x}>", unknown);
            offset + 1
        }
    }
}

/// Write `OFFSET  BB BB BB  ` prefix (offset + raw bytes column).
fn write_prefix(out: &mut String, bytecode: &Bytecode, offset: usize, size: usize) {
    let _ = write!(out, "{:04}  ", offset);
    let mut bytes = String::new();
    for i in 0..size {
        if i > 0 {
            bytes.push(' ');
        }
        let _ = write!(bytes, "{:02x}", bytecode.code[offset + i]);
    }
    let _ = write!(out, "{:<width$}  ", bytes, width = BYTES_COL_WIDTH);
}

fn simple(out: &mut String, name: &str, bytecode: &Bytecode, offset: usize) -> usize {
    write_prefix(out, bytecode, offset, 1);
    let _ = writeln!(out, "{}", name);
    offset + 1
}

fn constant(out: &mut String, name: &str, bytecode: &Bytecode, offset: usize) -> usize {
    write_prefix(out, bytecode, offset, 2);
    let idx = bytecode.code[offset + 1];
    let value = bytecode
        .constants
        .get(idx as usize)
        .map(format_constant)
        .unwrap_or_else(|| "<out of range>".to_string());
    let _ = writeln!(out, "{:<14} {}", name, value);
    offset + 2
}

fn jump(out: &mut String, name: &str, bytecode: &Bytecode, offset: usize) -> usize {
    write_prefix(out, bytecode, offset, 3);
    let lo = bytecode.code[offset + 1];
    let hi = bytecode.code[offset + 2];
    let delta = u16::from_le_bytes([lo, hi]) as usize;
    // The VM reads the 2-byte operand and then adds the delta, so the target
    // is the byte after the operand plus the delta.
    let target = offset + 3 + delta;
    let _ = writeln!(out, "{:<14} -> {:04}", name, target);
    offset + 3
}

/// Like `jump`, but for the backward `LOOP` opcode (VM subtracts the delta).
fn loop_jump(out: &mut String, name: &str, bytecode: &Bytecode, offset: usize) -> usize {
    write_prefix(out, bytecode, offset, 3);
    let lo = bytecode.code[offset + 1];
    let hi = bytecode.code[offset + 2];
    let delta = u16::from_le_bytes([lo, hi]) as usize;
    let target = (offset + 3).saturating_sub(delta);
    let _ = writeln!(out, "{:<14} -> {:04}", name, target);
    offset + 3
}

/// Format a `CLOSURE` instruction: `<fn_const> <count>` then `count` pairs of
/// `(is_local, index)` bytes describing each captured upvalue.
fn closure(out: &mut String, bytecode: &Bytecode, offset: usize) -> usize {
    let fn_idx = bytecode.code[offset + 1];
    let count = bytecode.code[offset + 2] as usize;
    let total = 3 + count * 2;
    write_prefix(out, bytecode, offset, total.min(BYTES_COL_WIDTH));
    let value = bytecode
        .constants
        .get(fn_idx as usize)
        .map(format_constant)
        .unwrap_or_else(|| "<out of range>".to_string());
    let _ = writeln!(out, "{:<14} {} upvals={}", "CLOSURE", value, count);
    offset + total
}

fn byte_operand(out: &mut String, name: &str, bytecode: &Bytecode, offset: usize) -> usize {
    write_prefix(out, bytecode, offset, 2);
    let idx = bytecode.code[offset + 1];
    let _ = writeln!(out, "{:<14} {}", name, idx);
    offset + 2
}

/// Format a name-constant + argc opcode (NEW, INVOKE, STATIC_INVOKE), whose
/// operands are a 1-byte constant index followed by a 1-byte argument count.
fn invoke(out: &mut String, name: &str, bytecode: &Bytecode, offset: usize) -> usize {
    write_prefix(out, bytecode, offset, 3);
    let idx = bytecode.code[offset + 1];
    let argc = bytecode.code[offset + 2];
    let value = bytecode
        .constants
        .get(idx as usize)
        .map(format_constant)
        .unwrap_or_else(|| "<out of range>".to_string());
    let _ = writeln!(out, "{:<14} {} ({})", name, value, argc);
    offset + 3
}

fn format_constant(c: &Constant) -> String {
    match c {
        Constant::Int(n) => format!("Int({})", n),
        Constant::Float(n) => format!("Float({})", n),
        Constant::Str(s) => format!("Str({:?})", s),
        Constant::Fn { entry, arity } => format!("Fn(@{}, /{})", entry, arity),
        Constant::Sym(s) => format!("Sym(#{})", s.id()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{BytecodeBuilder, Instruction};

    #[test]
    fn simple_opcodes() {
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::TRUE);
        b.emit(Instruction::NOT);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let dis = disassemble(&bc);
        // offset + raw byte + opcode name
        assert!(dis.contains("0000  05        TRUE"));
        assert!(dis.contains("0001  30        NOT"));
        assert!(dis.contains("0002  00        HALT"));
    }

    #[test]
    fn constant_opcode_inlines_value() {
        let mut b = BytecodeBuilder::new();
        b.emit_constant(Constant::Int(42));
        b.emit_constant(Constant::Str("hello".into()));
        b.emit(Instruction::ADD);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let dis = disassemble(&bc);
        // Raw bytes column should show opcode (01) + index byte.
        assert!(dis.contains("0000  01 00     CONST          Int(42)"));
        assert!(dis.contains("0002  01 01     CONST          Str(\"hello\")"));
    }

    #[test]
    fn jump_targets_are_resolved() {
        // if (true) {} else {} -> emits TRUE, JUMP_IF_FALSE, JUMP, HALT
        let mut b = BytecodeBuilder::new();
        b.emit(Instruction::TRUE);
        let jif = b.emit_jump(Instruction::JUMP_IF_FALSE);
        // then branch is empty
        let jmp = b.emit_jump(Instruction::JUMP);
        b.patch_jump(jif);
        // else branch is empty
        b.patch_jump(jmp);
        b.emit(Instruction::HALT);

        let bc = b.build();
        let dis = disassemble(&bc);
        // JUMP_IF_FALSE at 0001 skips the unconditional JUMP (3 bytes) to 0007.
        // Layout: 0000 TRUE, 0001 JUMP_IF_FALSE (3 bytes), 0004 JUMP (3 bytes), 0007 HALT.
        // Both jumps skip the empty branch and land at 0007.
        assert!(dis.contains("0001  51 03 00  JUMP_IF_FALSE  -> 0007"));
        assert!(dis.contains("0004  50 00 00  JUMP           -> 0007"));
    }

    #[test]
    fn unknown_opcode_does_not_panic() {
        let mut b = BytecodeBuilder::new();
        b.emit(0xEE);
        let bc = b.build();
        let dis = disassemble(&bc);
        assert!(dis.contains("<unknown 0xee>"));
    }
}
