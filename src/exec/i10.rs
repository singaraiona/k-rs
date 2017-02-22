use parse::ktree::{self, K};
use parse::parser::{self, Parser};
use parse::error::Error as ParseError;
use exec::error::Error as ExecError;
use std::rc::Rc;
use std::cell::RefCell;
use exec::env::Environment;
use parse::alloc::Arena;
use parse::vector::Vector;
use stacker;
use handle;

pub struct Interpreter {
    parser: Parser,
    arena: Arena,
}

impl Interpreter {
    fn add(&mut self, left: &K, right: &K, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a + b }),
            (&K::List { curry: true, values: ref a }, &K::Int { value: b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: b }, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: a }, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::List { curry: true, values: ref a }, &K::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for (x, y) in a.iter(&s1.arena.ktree).zip(b.iter(&s1.arena.ktree)) {
                    r.push(try!(s2.add(x, y, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn sub(&mut self, left: &K, right: &K, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a - b }),
            (&K::List { curry: true, values: ref a }, &K::Int { value: b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ktree) {
                    r.push(try!(s2.sub(x, &K::Int { value: b }, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ktree) {
                    r.push(try!(s2.sub(x, &K::Int { value: a }, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::List { curry: true, values: ref a }, &K::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for (x, y) in a.iter(&s1.arena.ktree).zip(b.iter(&s1.arena.ktree)) {
                    r.push(try!(s2.sub(x, y, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn prod(&mut self, left: &K, right: &K, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a * b }),
            (&K::List { curry: true, values: ref a }, &K::Int { value: b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ktree) {
                    r.push(try!(s2.prod(x, &K::Int { value: b }, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ktree) {
                    r.push(try!(s2.prod(x, &K::Int { value: a }, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::List { curry: true, values: ref a }, &K::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for (x, y) in a.iter(&s1.arena.ktree).zip(b.iter(&s1.arena.ktree)) {
                    r.push(try!(s2.prod(x, y, env.clone())));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn eq(&mut self, left: &K, right: &K, _: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Bool { value: a == b }),
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn cond(&mut self,
            c: &Vector<K, ktree::Id>,
            env: Rc<RefCell<Environment>>)
            -> Result<K, ExecError> {
        let (s1, s2) = handle::split(self);
        match c.as_slice(&s1.arena.ktree) {
            &[ref e, ref x, ref y] => {
                match try!(s2.run(&e, env.clone())) {
                    K::Bool { value: b } => {
                        if b {
                            return s2.run(&x, env.clone());
                        }
                        return s2.run(&y, env.clone());
                    }
                    _ => Err(ExecError::Condition),
                }
            }
            _ => Err(ExecError::Condition),
        }
    }

    fn call(&mut self,
            lambda: &K,
            cargs: &[K],
            env: Rc<RefCell<Environment>>)
            -> Result<K, ExecError> {
        match lambda {
            &K::Lambda { args: ref a, body: ref b } => {
                let e = Environment::new_child(env);
                for (n, v) in a.iter().zip(cargs) {
                    let x = try!(self.run(&v, e.clone()));
                    let _ = self.define(*n, &x, e.clone());
                }
                if stacker::remaining_stack() <= 8013672 {
                    return Err(ExecError::Stack);
                }
                let (s1, s2) = handle::split(self);
                return s1.run(s2.arena.ktree.deref(*b), e.clone());
            }
            _ => (),
        }
        Err(ExecError::Call)
    }

    fn apply(&mut self,
             lambda: &K,
             args: &[K],
             env: Rc<RefCell<Environment>>)
             -> Result<K, ExecError> {
        self.call(lambda, args, env.clone())
    }

    fn define(&mut self,
              id: u16,
              value: &K,
              env: Rc<RefCell<Environment>>)
              -> Result<K, ExecError> {
        let v = try!(self.run(value, env.clone()));
        env.borrow_mut().define(id, v.clone());
        Ok(v)
    }

    fn get(&mut self, id: u16, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match env.borrow().get(id) {
            Some(n) => Ok(n),
            None => Err(ExecError::Undefined),
        }
    }

    pub fn parse(&mut self, b: &[u8]) -> Result<K, ParseError> {
        self.parser.parse(b, &mut self.arena)
    }

    pub fn run(&mut self, node: &K, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match *node {
            K::Verb { kind: k, args: ref a } => {
                match k as char {
                    '+' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).run(&arg[0], env.clone()));
                        let y = try!(handle::from_raw(h).run(&arg[1], env.clone()));
                        return handle::from_raw(h).add(&x, &y, env.clone());
                    }
                    '-' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).run(&arg[0], env.clone()));
                        let y = try!(handle::from_raw(h).run(&arg[1], env.clone()));
                        return handle::from_raw(h).sub(&x, &y, env.clone());
                    }
                    '*' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).run(&arg[0], env.clone()));
                        let y = try!(handle::from_raw(h).run(&arg[1], env.clone()));
                        return handle::from_raw(h).prod(&x, &y, env.clone());
                    }
                    '=' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).run(&arg[0], env.clone()));
                        let y = try!(handle::from_raw(h).run(&arg[1], env.clone()));
                        return handle::from_raw(h).eq(&x, &y, env.clone());
                    }
                    '.' => {
                        let h = handle::into_raw(self);
                        let x = try!(handle::from_raw(h)
                            .run(a.get(0, &handle::from_raw(h).arena.ktree), env.clone()));
                        match a.get(1, &handle::from_raw(h).arena.ktree) {
                            &K::List { curry: true, values: ref v } => {
                                return handle::from_raw(h)
                                    .call(&x,
                                          v.as_slice(&mut handle::from_raw(h).arena.ktree),
                                          env.clone());
                            }
                            _ => {
                                return self.call(&x,
                                                 &a.as_slice(&handle::from_raw(h).arena.ktree)[1..],
                                                 env.clone())
                            }

                        }
                    }
                    '@' => {
                        let h = handle::into_raw(self);
                        let x = try!(handle::from_raw(h)
                            .run(a.get(0, &handle::from_raw(h).arena.ktree), env.clone()));
                        match a.get(1, &handle::from_raw(h).arena.ktree) {
                            &K::List { curry: true, values: ref v } => {
                                return handle::from_raw(h)
                                    .apply(&x,
                                           v.as_slice(&mut handle::from_raw(h).arena.ktree),
                                           env.clone());
                            }
                            _ => {
                                return handle::from_raw(h)
                                    .apply(&x,
                                           &a.as_slice(&handle::from_raw(h).arena.ktree)[1..],
                                           env.clone())
                            }
                        }
                    }
                    _ => (),
                };
            }
            K::Condition { list: ref c } => return self.cond(c, env.clone()),
            K::Nameref { id: n, value: ref v } => {
                let (s1, s2) = handle::split(self);
                let u = s1.arena.ktree.deref(*v);
                return s2.define(n, u, env.clone());
            }
            K::Name { value: n } => return self.get(n, env.clone()),
            K::Int { value: v } => return Ok(K::Int { value: v }),
            _ => return Ok(node.clone()),
        };
        Ok(K::Nil)
    }

    pub fn arena(&self) -> &Arena {
        &self.arena
    }
}

pub fn new() -> Interpreter {
    Interpreter {
        parser: parser::new(),
        arena: Arena::new(),
    }
}