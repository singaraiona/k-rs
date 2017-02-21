use std::mem;
use std::ops::Index;
use std::slice::Iter;
use parse::alloc::Arena;
use parse::vector::Vector;
use std::io::{stdout, Write};

#[derive(Debug, Clone)]
pub struct Args {
    args: [u16; 8],
    len: u8,
}

impl Args {
    pub fn new() -> Self {
        Args {
            args: [0u16; 8],
            len: 0,
        }
    }

    pub fn push(&mut self, a: u16) {
        // Warning: Without checking of capacity
        self.args[self.len()] = a;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn iter(&self) -> Iter<u16> {
        self.args.iter()
    }

    fn pp(&self, arena: &Arena) {
        let mut f = stdout();
        let _ = write!(f, "[");
        for i in 0..self.len() - 1 {
            let _ = write!(f, "{};", arena.id_name(self.args[i]));
        }
        let _ = write!(f, "{}", arena.id_name(self.args[self.len() - 1]));
        let _ = write!(f, "]");
    }
}

#[macro_export]
macro_rules! args {
    ( $( $x:expr ),* ) => {
        {
            let mut a = Args::new();
            $(
                a.push($x);
            )*
            a
        }
    };
}

impl Index<usize> for Args {
    type Output = u16;
    fn index(&self, i: usize) -> &Self::Output {
        &self.args[i]
    }
}

#[derive(Debug, Clone)]
pub enum K {
    Name { value: u16 },
    Bool { value: bool },
    Symbol { value: u16 },
    Verb { kind: u8, args: Vec<K> },
    Ioverb { fd: u8 },
    Int { value: i64 },
    Float { value: f64 },
    Lambda { args: Args, body: Box<K> },
    List { curry: bool, values: Vec<K> },
    Vector { curry: bool, values: Vector<K, Id> },
    Dict { keys: Vec<K>, values: Vec<K> },
    Nameref { id: u16, value: Box<K> },
    Adverb {
        kind: [u8; 2],
        left: Box<K>,
        verb: Box<K>,
        right: Box<K>,
    },
    Condition { list: Vec<K> },
    Nil,
}

impl K {
    pub fn find_names(&self, v: &mut Vec<u16>) -> usize {
        match *self {
            K::Name { value: n } => {
                v.push(n);
                1
            }
            K::Verb { kind: _, args: ref x } => x.iter().fold(0, |a, ref i| a + i.find_names(v)),
            K::Condition { list: ref x } => x.iter().fold(0, |a, ref i| a + i.find_names(v)),
            K::List { curry: _, values: ref x } => x.iter().fold(0, |a, ref i| a + i.find_names(v)),
            _ => 0,
        }
    }
}

impl PartialEq for K {
    fn eq(&self, other: &K) -> bool {
        mem::discriminant(self) == mem::discriminant(&other)
    }
}

pub fn pp(ktree: &K, arena: &Arena) {
    let mut f = stdout();
    match *ktree {
        K::Name { value: v } => {
            let _ = write!(f, "{}", arena.id_name(v));
        }
        K::Bool { value: ref v } => {
            let _ = write!(f, "{}b", *v as u8);
        }
        K::Symbol { value: v } => {
            let _ = write!(f, "`{}", arena.id_symbol(v));
        }
        K::Int { value: v } => {
            let _ = write!(f, "{}", v);
        }
        K::Float { value: v } => {
            let _ = write!(f, "{}", v);
        }
        K::Verb { kind: ref v, args: ref a } => {
            if a.len() > 0 {
                pp(&a[0], arena);
            }
            let _ = write!(f, "{}", *v as char);
            for i in 1..a.len() - 1 {
                pp(&a[i], arena);
            }
            pp(&a[a.len() - 1], arena);
        }

        K::Lambda { args: ref a, body: ref b } => {
            let _ = write!(f, "{{");
            a.pp(arena);
            pp(b, arena);
            let _ = write!(f, "}}");
        }
        K::List { curry: ref c, values: ref v } => {
            if !c {
                let _ = write!(f, "(");
                for i in 0..v.len() - 1 {
                    pp(&v[i], arena);
                    let _ = write!(f, ";");
                }
                pp(&v[v.len() - 1], arena);
                let _ = write!(f, ")");
            } else {
                for i in 0..v.len() - 1 {
                    pp(&v[i], arena);
                    let _ = write!(f, " ");
                }
                pp(&v[v.len() - 1], arena);
            }
        }
        // K::Dict { keys: ref k, values: ref v } => {
        //     write!(f, "[");
        //     for (key, val) in k[..k.len() - 1].iter().zip(v.iter()) {
        //         try!(write!(f, "{}:{};", key, val))
        //     }
        //     write!(f, "{}:{}]", k[k.len() - 1], v[v.len() - 1])
        // }
        // K::Condition { list: ref c } => {
        //     try!(write!(f, "$["));
        //     for i in 0..c.len() - 1 {
        //         try!(write!(f, "{};", c[i]))
        //     }
        //     write!(f, "{}]", c[c.len() - 1])
        //
        // }
        // K::Nil => Ok(()),
        _ => {
            let _ = write!(f, "nyi");
        }
    };
}

pub fn verb(c: char, args: Vec<K>) -> K {
    K::Verb {
        kind: c as u8,
        args: args,
    }
}

pub fn adverb(s: String, left: Box<K>, verb: Box<K>, right: Box<K>) -> K {
    let b = s.into_bytes();
    K::Adverb {
        kind: [b[0], b[1]],
        left: left,
        verb: verb,
        right: right,
    }
}


pub type Id = u64;