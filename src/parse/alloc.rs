use std::collections::HashMap;
use parse::arena::ArenaMem;
use parse::ast::{self, AST};

#[derive(Debug)]
pub struct Arena {
    pub names: HashMap<String, u16>,
    pub symbols: HashMap<String, u16>,
    pub ast: ArenaMem<AST, ast::Id>,
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            names: HashMap::new(),
            symbols: HashMap::new(),
            ast: ArenaMem::with_capacity(100),
        }
    }

    pub fn intern_symbol(&mut self, s: String) -> AST {
        let id = self.symbols.len() as u16;
        AST::Symbol { value: *self.symbols.entry(s).or_insert(id) }
    }

    pub fn intern_name(&mut self, s: String) -> AST {
        let id = self.names.len() as u16;
        AST::Name { value: *self.names.entry(s).or_insert(id) }
    }

    pub fn intern_name_id(&mut self, s: String) -> u16 {
        let id = self.names.len() as u16;
        *self.names.entry(s).or_insert(id)
    }

    pub fn name_id(&self, s: &str) -> u16 {
        match self.names.get(s) {
            Some(&id) => id,
            None => self.names.len() as u16,
        }
    }

    pub fn id_name(&self, id: u16) -> String {
        for (key, val) in self.names.iter() {
            if *val == id {
                return key.clone();
            }
        }
        "".to_string()
    }

    pub fn id_symbol(&self, id: u16) -> String {
        for (key, val) in self.symbols.iter() {
            if *val == id {
                return key.clone();
            }
        }
        "".to_string()
    }
}