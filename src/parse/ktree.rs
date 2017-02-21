use std::fmt::{self, Display};
use parse::error::Error as ParseError;
use exec::error::Error as ExecError;
use std::ops::Deref;
use std::mem;
use std::rc::Rc;
use std::cell::UnsafeCell;
use std::ops::Index;
use std::slice::Iter;
use exec::i10::Arena;
use std::io::{self, stdout, Write};

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

impl Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "["));
        for i in 0..self.len() - 1 {
            try!(write!(f, "{};", self.args[i]));
        }
        write!(f, "{}]", self.args[self.len() - 1])
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
    Dict { keys: Vec<K>, values: Vec<K> },
    Nameref { id: u16, value: Box<K> },
    Adverb {
        kind: String,
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
        K::Name { value: ref v } => {
            write!(f, "{}", v);
        }
        K::Bool { value: ref v } => {
            write!(f, "{}b", *v as u8);
        }
        K::Symbol { value: v } => {
            write!(f, "`{}", arena.id_symbol(v));
        }
        K::Int { value: v } => {
            write!(f, "{}", v);
        }
        K::Float { value: v } => {
            write!(f, "{}", v);
        }
        K::Verb { kind: ref v, args: ref a } => {
            if a.len() > 0 {
                pp(&a[0], arena);
            }
            write!(f, "{}", *v as char);
            for i in 1..a.len() - 1 {
                pp(&a[i], arena);
            }
            pp(&a[a.len() - 1], arena);
        }

        K::Lambda { args: ref a, body: ref b } => {
            write!(f, "{{{}", a);
            pp(b, arena);
            write!(f, "}}");
        }
        K::List { curry: ref c, values: ref v } => {
            if !c {
                write!(f, "(");
                for i in 0..v.len() - 1 {
                    pp(&v[i], arena);
                    write!(f, ";");
                }
                pp(&v[v.len() - 1], arena);
                write!(f, ")");
            } else {
                for i in 0..v.len() - 1 {
                    pp(&v[i], arena);
                    write!(f, " ");
                }
                pp(&v[v.len() - 1], arena);
            }
        }
        // K::Dict { keys: ref k, values: ref v } => {
        //     try!(write!(f, "["));
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
            write!(f, "nyi");
        }
    };
}

pub fn verb(c: char, args: Vec<K>) -> K {
    K::Verb {
        kind: c as u8,
        args: args,
    }
}

pub fn adverb(s: &str, left: Box<K>, verb: Box<K>, right: Box<K>) -> K {
    K::Adverb {
        kind: s.to_string(),
        left: left,
        verb: verb,
        right: right,
    }
}
