use crate::Symbol;

pub struct GlobalTable {
    names: Vec<Symbol>,
}

impl GlobalTable {
    pub fn new() -> Self {
        Self { names: Vec::new() }
    }

    pub fn define(&mut self, name: Symbol) -> Result<u8, String> {
        if self.resolve(name).is_some() {
            return Err("global already defined".to_string());
        }

        let idx = self.names.len();
        if idx > u8::MAX as usize {
            return Err("too many globals".to_string());
        }

        self.names.push(name);
        Ok(idx as u8)
    }

    pub fn resolve(&self, name: Symbol) -> Option<u8> {
        self.names.iter().position(|&n| n == name).map(|i| i as u8)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.names.len()
    }
}
