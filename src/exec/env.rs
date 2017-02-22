
use parse::ast;
use exec::otree::{self, Tree};

#[derive(Debug, Clone)]
pub struct Entry(u16, ast::Id);

#[derive(Debug)]
pub struct Environment {
    pub tree: Tree<Entry>,
}

impl Environment {
    pub fn new_root() -> Environment {
        let s = Tree::with_capacity(10000 as usize);
        Environment { tree: s }
    }

    pub fn last(&self) -> otree::Id {
        self.tree.last()
    }

    pub fn dump(&self) {
        self.tree.dump()
    }

    pub fn len(&self) -> (usize, usize) {
        self.tree.len()
    }

    pub fn new_child(&mut self, n: otree::Id) -> otree::Id {
        self.tree.append_node(n)
    }

    pub fn define(&mut self, key: u16, value: ast::Id) {
        self.tree.insert(Entry(key, value));
    }

    pub fn get(&self, key: u16, n: otree::Id) -> Option<(ast::Id, otree::Id)> {
        match self.tree.get(n, |e| e.0 == key) {
            Some(x) => Some(((x.0).1, x.1)),
            None => None,
        }
    }

    pub fn clean(&mut self) -> usize {
        self.tree.clean()
    }
}
