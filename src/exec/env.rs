use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use parse::ktree::K;

#[derive(PartialEq, Debug, Clone)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<u16, K>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Environment>> {
        let mut env = Environment {
            parent: None,
            values: HashMap::with_capacity(100),
        };
        Rc::new(RefCell::new(env))
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let env = Environment {
            parent: Some(parent),
            values: HashMap::with_capacity(100),
        };
        Rc::new(RefCell::new(env))
    }

    pub fn define(&mut self, key: u16, value: K) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: u16) -> Option<K> {
        match self.values.get(&key) {
            Some(val) => Some(val.clone()),
            None => {
                match self.parent {
                    Some(ref parent) => parent.borrow().get(key),
                    None => None,
                }
            }
        }
    }
}