//! String interning for efficient string storage and comparison.
//!
//! This module provides a string interner that maps strings to unique `Symbol` IDs.
//! This allows for:
//! - O(1) string comparison (compare u32 instead of string contents)
//! - Reduced memory usage (each unique string stored once)
//! - Faster hashing (hash a u32 instead of string)
//!
//! The `InternInner` implementation is based on the `intern-string` crate by Yagiz Nizipli,
//! licensed under the MIT License:
//!
//! Copyright 2024 Yagiz Nizipli
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use fxhash::{FxBuildHasher, FxHashMap};
use std::cell::RefCell;

/// A Symbol is a unique ID representing an interned string.
/// Two symbols are equal if and only if they represent the same string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

impl Symbol {
    /// Get the raw ID of this symbol (for debugging).
    #[inline]
    pub fn id(self) -> u32 {
        self.0
    }
}

/// Inner interner implementation.
struct InternInner {
    data: FxHashMap<&'static str, Symbol>,
    list: Vec<Box<str>>,
}

impl InternInner {
    fn new() -> Self {
        Self {
            data: FxHashMap::default(),
            list: Vec::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default()),
            list: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    fn intern<V: Into<String> + AsRef<str>>(&mut self, input: V) -> Symbol {
        if let Some(&id) = self.data.get(input.as_ref()) {
            return id;
        }

        let owned = input.into().into_boxed_str();

        let str_data = owned.as_ptr();
        let str_len = owned.len();

        let id = Symbol(self.list.len() as u32);
        self.list.push(owned);

        // SAFETY: we can do this because the allocations inside of a Box<str>
        // are stable, and so passing ownership to push does not change the
        // address.
        //
        // Additionally, because we have not touched the string since we created
        // these raw pointers ourselves, we know that it is valid UTF-8 and so
        // can skip that check as well.
        let k =
            unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(str_data, str_len)) };

        self.data.insert(k, id);
        id
    }

    #[inline]
    fn lookup(&self, id: Symbol) -> &str {
        &self.list[id.0 as usize]
    }

    #[inline]
    fn try_lookup(&self, id: Symbol) -> Option<&str> {
        self.list.get(id.0 as usize).map(|s| &**s)
    }

    fn len(&self) -> usize {
        self.list.len()
    }

    fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

/// String interner with interior mutability.
///
/// Stores a mapping from strings to unique Symbol IDs, ensuring each
/// unique string is stored only once. Uses `RefCell` for interior mutability,
/// allowing `intern` to be called with `&self`.
pub struct Interner {
    inner: RefCell<InternInner>,
}

impl std::fmt::Debug for Interner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("Interner")
            .field("count", &inner.len())
            .finish()
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl Interner {
    /// Create a new empty interner.
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(InternInner::new()),
        }
    }

    /// Create a new interner with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RefCell::new(InternInner::with_capacity(capacity)),
        }
    }

    /// Intern a string, returning its unique Symbol.
    ///
    /// If the string has been interned before, returns the existing Symbol.
    /// Otherwise, creates a new Symbol for this string.
    #[inline]
    pub fn intern(&self, name: &str) -> Symbol {
        self.inner.borrow_mut().intern(name)
    }

    /// Resolve a Symbol back to its string.
    ///
    /// Returns a cloned String for API compatibility.
    ///
    /// # Panics
    /// Panics if the symbol was not created by this interner.
    #[inline]
    pub fn resolve(&self, symbol: Symbol) -> String {
        self.inner.borrow().lookup(symbol).to_string()
    }

    /// Resolve a Symbol back to its string, returning None if invalid.
    #[inline]
    pub fn try_resolve(&self, symbol: Symbol) -> Option<String> {
        self.inner
            .borrow()
            .try_lookup(symbol)
            .map(|s| s.to_string())
    }

    /// Get the number of interned strings.
    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    /// Check if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_returns_same_symbol() {
        let interner = Interner::new();
        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_different_strings_different_symbols() {
        let interner = Interner::new();
        let s1 = interner.intern("hello");
        let s2 = interner.intern("world");
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_resolve() {
        let interner = Interner::new();
        let symbol = interner.intern("hello");
        assert_eq!(interner.resolve(symbol), "hello");
    }

    #[test]
    fn test_len() {
        let interner = Interner::new();
        assert_eq!(interner.len(), 0);
        interner.intern("a");
        assert_eq!(interner.len(), 1);
        interner.intern("b");
        assert_eq!(interner.len(), 2);
        interner.intern("a"); // duplicate
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_symbol_copy() {
        let interner = Interner::new();
        let s1 = interner.intern("test");
        let s2 = s1; // Copy
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_symbol_hash() {
        use std::collections::HashSet;
        let interner = Interner::new();
        let s1 = interner.intern("a");
        let s2 = interner.intern("b");
        let s3 = interner.intern("a");

        let mut set = HashSet::new();
        set.insert(s1);
        set.insert(s2);
        set.insert(s3);

        assert_eq!(set.len(), 2); // s1 and s3 are the same
    }

    #[test]
    fn test_with_capacity() {
        let interner = Interner::with_capacity(100);
        assert!(interner.is_empty());
        interner.intern("test");
        assert_eq!(interner.len(), 1);
    }
}
