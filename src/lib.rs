pub mod ast;
pub mod context;
pub mod interner;
mod parser;

mod tokeniser;
pub mod vm;

// Re-export interner types
pub use interner::{Interner, Symbol};

// Re-export context
pub use context::Context;

// Re-export AST types
pub use ast::{Expr, Literal, Operation, ParamVec, Program, Stmt};

// Re-export stack VM types
pub use vm::{
    AxeVM, Bytecode, BytecodeBuilder, CompileError, Compiler, FileLoader, ModuleLoader,
    Obj as VMObj, RuntimeError, Value as VMValue, disassemble, disassemble_instruction,
};

pub use parser::{ParseError, Parser};

// Re-export smallvec for tests and users
pub use smallvec;
