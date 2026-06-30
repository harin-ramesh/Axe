use crate::Symbol;

pub struct GlobalTable {
    names: Vec<Symbol>
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
        self.names.iter().position(|&n| n==name).map(|i| i as u8)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.names.len()
    }
}

struct Local {
    name: Symbol,
    depth: usize,
}

impl Local {
    pub fn new(name: Symbol, depth: usize) -> Self {
        Self {
            name,
            depth
        }
    } 
}

pub struct LocalTable {
    locals: Vec<Local>
}

impl LocalTable {
    pub fn new() -> Self {
        Self { locals: Vec::new() }
    }

    pub fn define(&mut self, name: Symbol, depth: usize) -> Result<u8, String> {
        if self.resolve(name, depth).is_some() {
            return Err("global already defined".to_string());
        }

        let idx = self.locals.len();
        if idx > u8::MAX as usize {
            return Err("too many globals".to_string());
        }

        self.locals.push(Local::new(name, depth));
        Ok(idx as u8)
    }

    pub fn resolve(&self, name: Symbol, depth: usize) -> Option<u8> {
        self.locals.iter().position(|n| n.name==name && n.depth <= depth).map(|i| i as u8)
    }

    pub fn pop_scope(&mut self, depth: usize) -> usize {
        let mut count = 0;
        while let Some(local) = self.locals.last() {
            if local.depth >= depth {
                self.locals.pop();
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.locals.len()
    }
}

