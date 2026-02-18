pub mod ast;
pub mod context;
pub mod interner;
mod parser;
pub mod stack_vm;
mod tokeniser;
pub mod transformer;
pub mod tree_walker;

// Re-export interner types
pub use interner::{Interner, Symbol};

// Re-export context
pub use context::Context;

// Re-export AST types
pub use ast::{Expr, Literal, Operation, Program, Stmt};

// Re-export tree-walker interpreter types
pub use tree_walker::{
    EnvRef, Environment, EvalSignal, Locals, ResolvedLocation, Resolver, TreeWalker, Value,
};

// Re-export for backwards compatibility
pub use tree_walker::TreeWalker as Axe;

// Re-export stack VM types
pub use stack_vm::{AxeVM, Chunk, Compiler, Value as VMValue};

pub use parser::Parser;
pub use transformer::Transformer;
