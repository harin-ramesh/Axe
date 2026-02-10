//! Tree-walking interpreter for the Axe language.
//!
//! This module provides a straightforward AST interpreter that directly
//! executes the parsed program. It serves as a reference implementation
//! and backup for a future bytecode VM.

mod builtins;
mod environment;
mod tree_walker;
mod value;
mod instructions;
mod vm;

pub use environment::{EnvRef, Environment};
pub use tree_walker::{EvalSignal, TreeWalker};
pub use value::Value;
