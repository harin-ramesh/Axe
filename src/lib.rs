pub mod ast;
pub mod context;
pub mod interner;
mod parser;

mod tokeniser;
pub mod transformer;
pub mod tree_walker;
pub mod vm;

// Re-export interner types
pub use interner::{Interner, Symbol};

// Re-export context
pub use context::Context;

// Re-export AST types
pub use ast::{Expr, Literal, Operation, ParamVec, Program, Stmt};

// Re-export tree-walker interpreter types
pub use tree_walker::{
    EnvRef, Environment, EvalSignal, Locals, ResolvedLocation, Resolver, TreeWalker, Value,
};

// Re-export for backwards compatibility
pub use tree_walker::TreeWalker as Axe;

// Re-export stack VM types
pub use vm::{
    AxeVM, Bytecode, BytecodeBuilder, Compiler, Obj as VMObj, Value as VMValue, disassemble,
    disassemble_instruction,
};

pub use parser::{ParseError, Parser};
pub use transformer::Transformer;

// Re-export smallvec for tests and users
pub use smallvec;
