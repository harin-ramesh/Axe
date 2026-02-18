//! Tree-walking interpreter for the Axe language.
//!
//! This module provides a straightforward AST interpreter that directly
//! executes the parsed program. It serves as a reference implementation
//! and backup for the bytecode VM.

mod builtins;
mod environment;
mod interpreter;
mod resolver;
mod value;

pub use environment::{EnvRef, Environment};
pub use interpreter::{EvalSignal, TreeWalker};
pub use resolver::{Locals, ResolvedLocation, Resolver};
pub use value::Value;
