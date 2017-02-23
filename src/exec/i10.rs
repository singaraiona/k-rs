use parse::ast::{self, AST};
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

    fn add(&mut self, left: &AST, right: &AST, id: otree::Id) -> Result<AST, ExecError> {
        match (left, right) {
            (&AST::Int { value: a }, &AST::Int { value: b }) => {
                return Ok(AST::Int { value: a + b })
            }
            (&AST::List { curry: true, values: ref a }, &AST::Int { value: b }) => {
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ast) {
                    r.push(try!(s2.add(x, &AST::Int { value: b }, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            (&AST::Int { value: a }, &AST::List { curry: true, values: ref b }) => {
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ast) {
                    r.push(try!(s2.add(x, &AST::Int { value: a }, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            (&AST::List { curry: true, values: ref a },
             &AST::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for (x, y) in a.iter(&s1.arena.ast).zip(b.iter(&s1.arena.ast)) {
                    r.push(try!(s2.add(x, y, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn sub(&mut self, left: &AST, right: &AST, id: otree::Id) -> Result<AST, ExecError> {
        match (left, right) {
            (&AST::Int { value: a }, &AST::Int { value: b }) => {
                return Ok(AST::Int { value: a - b })
            }
            (&AST::List { curry: true, values: ref a }, &AST::Int { value: b }) => {
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ast) {
                    r.push(try!(s2.add(x, &AST::Int { value: b }, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            (&AST::Int { value: a }, &AST::List { curry: true, values: ref b }) => {
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ast) {
                    r.push(try!(s2.add(x, &AST::Int { value: a }, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            (&AST::List { curry: true, values: ref a },
             &AST::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for (x, y) in a.iter(&s1.arena.ast).zip(b.iter(&s1.arena.ast)) {
                    r.push(try!(s2.add(x, y, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn prod(&mut self, left: &AST, right: &AST, id: otree::Id) -> Result<AST, ExecError> {
        match (left, right) {
            (&AST::Int { value: a }, &AST::Int { value: b }) => {
                return Ok(AST::Int { value: a * b })
            }
            (&AST::List { curry: true, values: ref a }, &AST::Int { value: b }) => {
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in a.iter(&s1.arena.ast) {
                    r.push(try!(s2.add(x, &AST::Int { value: b }, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            (&AST::Int { value: a }, &AST::List { curry: true, values: ref b }) => {
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for x in b.iter(&s1.arena.ast) {
                    r.push(try!(s2.add(x, &AST::Int { value: a }, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            (&AST::List { curry: true, values: ref a },
             &AST::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<AST> = Vec::new();
                let (s1, s2) = handle::split(self);
                for (x, y) in a.iter(&s1.arena.ast).zip(b.iter(&s1.arena.ast)) {
                    r.push(try!(s2.add(x, y, id)));
                }
                return Ok(ast::list(true, &mut s1.arena.ast, r));
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn eq(&mut self, left: &AST, right: &AST, _: otree::Id) -> Result<AST, ExecError> {
        match (left, right) {
            (&AST::Int { value: a }, &AST::Int { value: b }) => {
                return Ok(AST::Bool { value: a == b })
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn cond(&mut self, c: &Vector<AST, ast::Id>, id: otree::Id) -> Result<AST, ExecError> {
        let (s1, s2) = handle::split(self);
        match c.as_slice(&s1.arena.ast) {
            &[ref e, ref x, ref y] => {
                match try!(s2.exec(&e, id)) {
                    AST::Bool { value: b } => {
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

    fn call(&mut self, lambda: &AST, cargs: &[AST], id: otree::Id) -> Result<AST, ExecError> {
        match lambda {
            &AST::Lambda { args: ref a, body: ref b } => {
                let e = self.env.new_child(id);
                for (n, v) in a.iter().zip(cargs) {
                    let x = try!(self.exec(&v, e));
                    let _ = self.define(*n, &x, e);
                }
                if stacker::remaining_stack() <= 8013672 {
                    return Err(ExecError::Stack);
                }
                let (s1, s2) = handle::split(self);
                let u = s2.arena.ast.deref(*b);
                return s1.exec(u, e);
            }
            _ => (),
        }
        Err(ExecError::Call)
    }

    fn apply(&mut self, lambda: &AST, args: &[AST], id: otree::Id) -> Result<AST, ExecError> {
        self.call(lambda, args, id)
    }

    fn define(&mut self, key: u16, value: &AST, id: otree::Id) -> Result<ast::Id, ExecError> {
        let v = try!(self.exec(value, id));
        let u = self.store(v);
        self.env.define(key, u);
        Ok(u)
    }

    fn define_id(&mut self, key: u16, value: ast::Id) -> Result<ast::Id, ExecError> {
        self.env.define(key, value);
        Ok(value)
    }

    fn get(&mut self, key: u16, id: otree::Id) -> Result<&AST, ExecError> {
        match self.env.get(key, id) {
            Some((n, _)) => Ok(self.arena.ast.deref(n)),
            None => Err(ExecError::Undefined),
        }
    }

    #[inline]
    fn store(&mut self, ast: AST) -> ast::Id {
        self.arena.ast.push(ast)
    }

    pub fn parse(&mut self, b: &[u8]) -> Result<AST, ParseError> {
        self.parser.parse(b, &mut self.arena)
    }

    pub fn run(&mut self, node: &AST) -> Result<AST, ExecError> {
        let id = self.env.last();
        self.exec(node, id)
    }

    fn exec(&mut self, node: &AST, id: otree::Id) -> Result<AST, ExecError> {
        match *node {
            AST::Verb { kind: k, args: ref a } => {
                match k as char {
                    '+' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ast);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).add(&x, &y, id);
                    }
                    '-' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ast);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).sub(&x, &y, id);
                    }
                    '*' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ast);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).prod(&x, &y, id);
                    }
                    '=' => {
                        let h = handle::into_raw(self);
                        let arg = a.as_slice(&handle::from_raw(h).arena.ast);
                        let x = try!(handle::from_raw(h).exec(&arg[0], id));
                        let y = try!(handle::from_raw(h).exec(&arg[1], id));
                        return handle::from_raw(h).eq(&x, &y, id);
                    }
                    '.' => {
                        let h = handle::into_raw(self);
                        let x = try!(handle::from_raw(h)
                            .exec(a.get(0, &handle::from_raw(h).arena.ast), id));
                        match a.get(1, &handle::from_raw(h).arena.ast) {
                            &AST::List { curry: true, values: ref v } => {
                                return handle::from_raw(h)
                                    .call(&x, v.as_slice(&mut handle::from_raw(h).arena.ast), id);
                            }
                            _ => {
                                return self.call(&x,
                                                 &a.as_slice(&handle::from_raw(h).arena.ast)[1..],
                                                 id)
                            }

                        }
                    }
                    '@' => {
                        let h = handle::into_raw(self);
                        let x = try!(handle::from_raw(h)
                            .exec(a.get(0, &handle::from_raw(h).arena.ast), id));
                        match a.get(1, &handle::from_raw(h).arena.ast) {
                            &AST::List { curry: true, values: ref v } => {
                                return handle::from_raw(h)
                                    .apply(&x, v.as_slice(&mut handle::from_raw(h).arena.ast), id);
                            }
                            _ => {
                                return handle::from_raw(h)
                                    .apply(&x, &a.as_slice(&handle::from_raw(h).arena.ast)[1..], id)
                            }
                        }
                    }
                    _ => (),
                };
            }
            AST::Condition { list: ref c } => return self.cond(c, id),
            AST::Nameref { name: n, value: v } => {
                let (s1, s2) = handle::split(self);
                let _ = try!(s2.define_id(n, v));
                return Ok(*s1.arena.ast.deref(v));
            }
            AST::Name { value: n } => {
                let u = try!(self.get(n, id));
                return Ok(*u);
            }
            AST::Int { value: v } => return Ok(AST::Int { value: v }),
            AST::List { curry: c, values: v } => {
                let (s1, s2) = handle::split(self);
                for u in v.as_slice_mut(&mut s1.arena.ast) {
                    *u = try!(s2.exec(u, id));
                }
                return Ok(AST::List {
                    curry: c,
                    values: v,
                });
            } 
            AST::Sequence { values: v } => {
                let (s1, s2) = handle::split(self);
                for u in v.as_slice_mut(&mut s1.arena.ast) {
                    *u = try!(s2.exec(u, id));
                }
                return Ok(*v.get(v.len() - 1, &s2.arena.ast));
            }
            _ => return Ok(*node),
        };
        Ok(AST::Nil)
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