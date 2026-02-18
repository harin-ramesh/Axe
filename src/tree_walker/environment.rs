//! Environment for variable bindings.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interner::Symbol;

use super::value::Value;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    records: HashMap<Symbol, Value>,
    parent: Option<EnvRef>,
}

impl Environment {
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Self {
            records: HashMap::new(),
            parent: None,
        }))
    }

    pub fn extend(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            records: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn get(&self, name: Symbol) -> Option<Value> {
        self.records
            .get(&name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.borrow().get(name))
    }

    pub fn set(&mut self, name: Symbol, value: Value) {
        self.records.insert(name, value);
    }

    pub fn update(&mut self, name: Symbol, value: Value) -> Result<(), &'static str> {
        if self.records.contains_key(&name) {
            self.records.insert(name, value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().update(name, value)
        } else {
            Err("undefined variable")
        }
    }

    /// Get a variable at a specific depth (0 = this scope, 1 = parent, etc.)
    /// This avoids walking the scope chain when we already know where the variable is.
    pub fn get_at(&self, depth: usize, name: Symbol) -> Option<Value> {
        if depth == 0 {
            self.records.get(&name).cloned()
        } else {
            self.parent.as_ref()?.borrow().get_at(depth - 1, name)
        }
    }

    /// Update a variable at a specific depth.
    pub fn set_at(&mut self, depth: usize, name: Symbol, value: Value) {
        if depth == 0 {
            self.records.insert(name, value);
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set_at(depth - 1, name, value);
        }
    }

    #[allow(dead_code)]
    pub fn exists_in_current_scope(&self, name: Symbol) -> bool {
        self.records.contains_key(&name)
    }

    #[allow(dead_code)]
    pub fn exists_in_any_scope(&self, name: Symbol) -> bool {
        self.records.contains_key(&name)
            || self
                .parent
                .as_ref()
                .is_some_and(|p| p.borrow().exists_in_any_scope(name))
    }
}
