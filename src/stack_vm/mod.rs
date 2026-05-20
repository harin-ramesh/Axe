//! Stack-based bytecode virtual machine for the Axe language.
//!
//! This module provides a bytecode compiler and VM that executes compiled
//! programs. It's more efficient than the tree-walker for repeated execution.

mod compiler;
mod disassembler;
mod instructions;
mod vm;

pub use compiler::Compiler;
pub use disassembler::{disassemble_chunk, disassemble_instruction};
pub use instructions::Instruction;
pub use vm::{AxeVM, Chunk, Obj, Value};
