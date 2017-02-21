use parse::ktree::K;
use parse::parser::Parser;
use parse::error::Error as ParseError;
use exec::error::Error as ExecError;
use std::rc::Rc;
use std::cell::RefCell;
use exec::env::Environment;
use stacker;

pub struct Interpreter {
    parser: Parser,
}

impl Interpreter {
    fn add(&mut self, left: &K, right: &K, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a + b }),
            (&K::List { curry: true, values: ref a }, &K::Int { value: b }) => {
                let mut r: Vec<K> = Vec::new();
                for x in a.iter() {
                    r.push(try!(self.add(x, &K::Int { value: b }, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                for x in b.iter() {
                    r.push(try!(self.add(x, &K::Int { value: a }, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
            }
            (&K::List { curry: true, values: ref a }, &K::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<K> = Vec::new();
                for (x, y) in a.iter().zip(b.iter()) {
                    r.push(try!(self.add(x, y, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
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
                for x in a.iter() {
                    r.push(try!(self.sub(x, &K::Int { value: b }, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                for x in b.iter() {
                    r.push(try!(self.sub(x, &K::Int { value: a }, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
            }
            (&K::List { curry: true, values: ref a }, &K::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<K> = Vec::new();
                for (x, y) in a.iter().zip(b.iter()) {
                    r.push(try!(self.sub(x, y, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
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
                for x in a.iter() {
                    r.push(try!(self.prod(x, &K::Int { value: b }, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
            }
            (&K::Int { value: a }, &K::List { curry: true, values: ref b }) => {
                let mut r: Vec<K> = Vec::new();
                for x in b.iter() {
                    r.push(try!(self.prod(x, &K::Int { value: a }, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
            }
            (&K::List { curry: true, values: ref a }, &K::List { curry: true, values: ref b }) => {
                if a.len() != b.len() {
                    return Err(ExecError::Length);
                }
                let mut r: Vec<K> = Vec::new();
                for (x, y) in a.iter().zip(b.iter()) {
                    r.push(try!(self.prod(x, y, env.clone())));
                }
                return Ok(K::List {
                    curry: true,
                    values: r,
                });
            }
            _ => (),
        };
        Err(ExecError::Type)
    }

    fn eq(&mut self, left: &K, right: &K, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match (left, right) {
            (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Bool { value: a == b }),
            _ => (),
        };
        // println!("EQ: {:?} {:?}", left, right);
        Err(ExecError::Type)
    }

    fn cond(&mut self, c: &[K], env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match c {
            &[ref e, ref x, ref y] => {
                match try!(self.run(&e, env.clone())) {
                    K::Bool { value: b } => {
                        if b {
                            return self.run(&x, env.clone());
                        }
                        return self.run(&y, env.clone());
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
                    self.define(n, &x, e.clone());
                }
                if stacker::remaining_stack() <= 8013672 {
                    return Err(ExecError::Stack);
                }
                return self.run(b, e.clone());
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
              name: &str,
              value: &K,
              env: Rc<RefCell<Environment>>)
              -> Result<K, ExecError> {
        let v = try!(self.run(value, env.clone()));
        env.borrow_mut().define(name.to_string(), v.clone());
        Ok(v)
    }

    fn get(&mut self, name: &str, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        match env.borrow().get(name) {
            Some(n) => Ok(n),
            None => Err(ExecError::Undefined),
        }
    }

    pub fn run(&mut self, k: &K, env: Rc<RefCell<Environment>>) -> Result<K, ExecError> {
        // println!("RUN: {:?}", k);
        match *k {
            K::Verb { kind: ref k, args: ref a } => {
                match &k[..] {
                    "+" => {
                        let x = try!(self.run(&a[0], env.clone()));
                        let y = try!(self.run(&a[1], env.clone()));
                        return self.add(&x, &y, env.clone());
                    }
                    "-" => {
                        let x = try!(self.run(&a[0], env.clone()));
                        let y = try!(self.run(&a[1], env.clone()));
                        return self.sub(&x, &y, env.clone());
                    }
                    "*" => {
                        let x = try!(self.run(&a[0], env.clone()));
                        let y = try!(self.run(&a[1], env.clone()));
                        return self.prod(&x, &y, env.clone());
                    }
                    "=" => {
                        let x = try!(self.run(&a[0], env.clone()));
                        let y = try!(self.run(&a[1], env.clone()));
                        return self.eq(&x, &y, env.clone());
                    }
                    "." => {
                        let x = try!(self.run(&a[0], env.clone()));
                        match &a[1] {
                            &K::List { curry: true, values: ref v } => {
                                return self.call(&x, &v[..], env.clone())
                            }
                            _ => return self.call(&x, &a[1..], env.clone()),
                        }
                    }
                    "@" => {
                        let x = try!(self.run(&a[0], env.clone()));
                        match &a[1] {
                            &K::List { curry: true, values: ref v } => {
                                return self.apply(&x, &v[..], env.clone())
                            }
                            _ => return self.apply(&x, &a[1..], env.clone()),                        
                        }
                    }
                    _ => (),
                };
            }
            K::Condition { list: ref c } => return self.cond(c, env.clone()),
            K::Nameref { name: ref n, value: ref v } => return self.define(&n[..], v, env.clone()),
            K::Name { value: ref n } => return self.get(n, env.clone()),        
            K::Int { value: v } => return Ok(K::Int { value: v }),
            _ => return Ok(k.clone()),
        };
        Ok(K::Nil)
    }
}

pub fn new() -> Interpreter {
    Interpreter { parser: Parser::new() }
}