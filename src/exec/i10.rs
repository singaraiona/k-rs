use parse::ktree::{self, K};
use parse::parser::{self, Parser};
use parse::error::Error as ParseError;
use exec::error::Error as ExecError;
use exec::env::Environment;
use parse::alloc::Arena;
use parse::vector::Vector;
use exec::otree;
use stacker;
use handle;

pub struct Interpreter {
    parser: Parser,
    arena: Arena,
    env: Environment,
}

impl Interpreter {
    pub fn gc(&mut self) {
        self.env.clean();
    }

    fn add(&mut self, left: &K, right: &K, id: otree::Id) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a + b }),
            (&K::List { curry: true, values: ref a }, &K::Int { value: b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: b }, id)));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: a }, id)));
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
                    r.push(try!(s2.add(x, y, id)));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn sub(&mut self, left: &K, right: &K, id: otree::Id) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a - b }),
            (&K::List { curry: true, values: ref a }, &K::Int { value: b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: b }, id)));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: a }, id)));
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
                    r.push(try!(s2.add(x, y, id)));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn prod(&mut self, left: &K, right: &K, id: otree::Id) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a * b }),
            (&K::List { curry: true, values: ref a }, &K::Int { value: b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: b }, id)));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ktree) {
                    r.push(try!(s2.add(x, &K::Int { value: a }, id)));
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
                    r.push(try!(s2.add(x, y, id)));
                }
                return Ok(ktree::list(true, &mut s1.arena.ktree, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn eq(&mut self, left: &K, right: &K, _: otree::Id) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Bool { value: a == b }),
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn cond(&mut self, c: &Vector<K, ktree::Id>, id: otree::Id) -> Result<K, ExecError> {
        let (s1, s2) = handle::split(self);
        match c.as_slice(&s1.arena.ktree) {
            &[ref e, ref x, ref y] => {
                match try!(s2.exec(&e, id)) {
                    K::Bool { value: b } => {
                        if b {
                            return s2.exec(&x, id);
                        }
                        return s2.exec(&y, id);
                    }
                    _ => Err(ExecError::Condition),
                }
            }
            _ => Err(ExecError::Condition),
        }
    }

    fn call(&mut self, lambda: &K, cargs: &[K], id: otree::Id) -> Result<K, ExecError> {
        match lambda {
            &K::Lambda { args: ref a, body: ref b } => {
                let e = self.env.new_child(id);
                for (n, v) in a.iter().zip(cargs) {
                    let x = try!(self.exec(&v, e));
                    let _ = self.define(*n, &x, e);
                }
                if stacker::remaining_stack() <= 8013672 {
                    return Err(ExecError::Stack);
                }
                let (s1, s2) = handle::split(self);
                let u = s2.arena.ktree.deref(*b);
                return s1.exec(u, e);
            }
            _ => (),
        }
        Err(ExecError::Call)
    }

    fn apply(&mut self, lambda: &K, args: &[K], id: otree::Id) -> Result<K, ExecError> {
        self.call(lambda, args, id)
    }

    fn define(&mut self, key: u16, value: &K, id: otree::Id) -> Result<ktree::Id, ExecError> {
        let v = try!(self.exec(value, id));
        let u = self.store(v);
        self.env.define(key, u);
        Ok(u)
    }

    fn define_id(&mut self, key: u16, value: ktree::Id) -> Result<ktree::Id, ExecError> {
        self.env.define(key, value);
        Ok(value)
    }

    fn get(&mut self, key: u16, id: otree::Id) -> Result<&K, ExecError> {
        match self.env.get(key, id) {
            Some((n, _)) => Ok(self.arena.ktree.deref(n)),
            None => Err(ExecError::Undefined),
        }
    }

    #[inline]
    fn store(&mut self, k: K) -> ktree::Id {
        self.arena.ktree.push(k)
    }

    pub fn parse(&mut self, b: &[u8]) -> Result<K, ParseError> {
        self.parser.parse(b, &mut self.arena)
    }

    pub fn run(&mut self, node: &K) -> Result<K, ExecError> {
        let id = self.env.last();
        self.exec(node, id)
    }

    fn exec(&mut self, node: &K, id: otree::Id) -> Result<K, ExecError> {
        match *node {
            K::Verb { kind: k, args: ref a } => {
                match k as char {
                    '+' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).add(&x, &y, id);
                    }
                    '-' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).sub(&x, &y, id);
                    }
                    '*' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).prod(&x, &y, id);
                    }
                    '=' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ktree);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).eq(&x, &y, id);
                    }
                    '.' => {
                        let h = handle::into_raw(self);
                        let x = try!(handle::from_raw(h)
                            .exec(a.get(0, &handle::from_raw(h).arena.ktree), id));
                        match a.get(1, &handle::from_raw(h).arena.ktree) {
                            &K::List { curry: true, values: ref v } => {
                                return handle::from_raw(h)
                                    .call(&x, v.as_slice(&mut handle::from_raw(h).arena.ktree), id);
                            }
                            _ => {
                                return self.call(&x,
                                                 &a.as_slice(&handle::from_raw(h).arena.ktree)[1..],
                                                 id)
                            }

                        }
                    }
                    '@' => {
                        let h = handle::into_raw(self);
                        let x = try!(handle::from_raw(h)
                            .exec(a.get(0, &handle::from_raw(h).arena.ktree), id));
                        match a.get(1, &handle::from_raw(h).arena.ktree) {
                            &K::List { curry: true, values: ref v } => {
                                return handle::from_raw(h)
                                    .apply(&x,
                                           v.as_slice(&mut handle::from_raw(h).arena.ktree),
                                           id);
                            }
                            _ => {
                                return handle::from_raw(h)
                                    .apply(&x,
                                           &a.as_slice(&handle::from_raw(h).arena.ktree)[1..],
                                           id)
                            }
                        }
                    }
                    _ => (),
                };
            }
            K::Condition { list: ref c } => return self.cond(c, id),
            K::Nameref { name: n, value: v } => {
                let (s1, s2) = handle::split(self);
                let _ = try!(s2.define_id(n, v));
                return Ok(s1.arena.ktree.deref(v).clone());
            }
            K::Name { value: n } => {
                let u = try!(self.get(n, id));
                return Ok(u.clone());
            }
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
        env: Environment::new_root(),
    }
}