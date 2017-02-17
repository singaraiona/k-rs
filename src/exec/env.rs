use std::collections::HashMap;
use parse::ktree::K;

pub struct Environment {
    env: HashMap<String, K>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { env: HashMap::new() }
    }

    pub fn define(&mut self, name: &str, value: &K) {
        self.env.insert(name.to_string(), value.clone());
    }

    pub fn get(&self, name: &str) -> Option<K> {
        match self.env.get(name) {
            Some(n) => Some(n.clone()),
            None => None,
        }
    }
}