//! Environment for variable bindings.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::Value;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    records: HashMap<String, Value>,
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

    pub fn get(&self, name: &str) -> Option<Value> {
        self.records
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.borrow().get(name))
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.records.insert(name, value);
    }

    pub fn update(&mut self, name: String, value: Value) -> Result<(), &'static str> {
        // Search for the variable in current scope
        if self.records.contains_key(&name) {
            self.records.insert(name, value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            // Search in parent scope
            parent.borrow_mut().update(name, value)
        } else {
            // Variable not found in any scope
            Err("undefined variable")
        }
    }

    #[allow(dead_code)]
    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        self.records.contains_key(name)
    }

    #[allow(dead_code)]
    pub fn exists_in_any_scope(&self, name: &str) -> bool {
        self.records.contains_key(name)
            || self
                .parent
                .as_ref()
                .map_or(false, |p| p.borrow().exists_in_any_scope(name))
    }
}
