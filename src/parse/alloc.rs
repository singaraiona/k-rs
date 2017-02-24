use std::collections::HashMap;
use parse::arena::ArenaMem;
use parse::ast::{self, AST};

pub struct Arena {
    pub names: HashMap<String, u16>,
    pub symbols: HashMap<String, u16>,
    pub ast: ArenaMem<AST, ast::Id>,
    pub natives: HashMap<u16, u8>,
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            names: HashMap::new(),
            symbols: HashMap::new(),
            ast: ArenaMem::with_capacity(100),
            natives: HashMap::new(),
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

    pub fn native_id(&self, s: &str) -> Option<u16> {
        match self.names.get(s) {
            Some(id) => {
                if self.natives.get(id).is_some() {
                    Some(*id)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn native_id_id(&self, id: u16) -> Option<u8> {
        self.natives.get(&id).map(|x| *x)
    }

    pub fn add_native(&mut self, name: String, funid: u8) {
        let id = self.intern_name_id(name);
        self.natives.insert(id, funid);
    }

    pub fn has_name(&self, s: &str) -> bool {
        self.names.get(s).is_some()
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