//! Shared context passed through the compilation and execution pipeline.
//!
//! The Context holds shared resources like the string interner that need
//! to be accessible throughout parsing, transformation, and interpretation.

use crate::interner::{Interner, Symbol};

/// Shared context for the Axe interpreter.
///
/// This struct holds resources that are shared across all phases of
/// compilation and execution.
#[derive(Debug)]
pub struct Context {
    /// The string interner for efficient string handling.
    pub interner: Interner,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new context with a fresh interner.
    pub fn new() -> Self {
        Self {
            interner: Interner::new(),
        }
    }

    /// Convenience method to intern a string.
    #[inline]
    pub fn intern(&self, s: &str) -> Symbol {
        self.interner.intern(s)
    }

    /// Convenience method to resolve a symbol.
    #[inline]
    pub fn resolve(&self, symbol: Symbol) -> String {
        self.interner.resolve(symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = Context::new();
        assert!(ctx.interner.is_empty());
    }

    #[test]
    fn test_context_intern_resolve() {
        let ctx = Context::new();
        let sym = ctx.intern("hello");
        assert_eq!(ctx.resolve(sym), "hello");
    }
}
