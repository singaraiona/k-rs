use std::fmt::{self, Display};
use parse::error::Error;
use std::ops::Deref;
use std::mem;

#[derive(Debug, Clone)]
pub enum K {
    Name { value: String },
    Symbol { value: String },
    Verb { kind: String, args: Vec<K> },
    Ioverb { fd: u8 },
    Int { value: i64 },
    Float { value: f64 },
    Lambda { args: Vec<String>, body: Box<K> },
    List { values: Vec<K> },
    Dict { keys: Vec<K>, values: Vec<K> },
    Nameref { name: String, value: Box<K> },
    Adverb {
        kind: String,
        left: Box<K>,
        verb: Box<K>,
        right: Box<K>,
    },
    Condition { list: Vec<K> },
    Internal { kind: String },
    Nil,
}

impl K {
    pub fn find_names(&self, v: &mut Vec<String>) -> usize {
        match *self {
            K::Name { value: ref n } => {
                v.push(n.clone());
                1
            }
            K::Verb { kind: _, args: ref x } => x.iter().fold(0, |a, ref i| a + i.find_names(v)),
            K::Condition { list: ref x } => x.iter().fold(0, |a, ref i| a + i.find_names(v)),
            K::List { values: ref x } => x.iter().fold(0, |a, ref i| a + i.find_names(v)),
            _ => 0,
        }
    }
}

impl PartialEq for K {
    fn eq(&self, other: &K) -> bool {
        mem::discriminant(self) == mem::discriminant(&other)
    }
}

impl Display for K {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            K::Name { value: ref v } => write!(f, "{}", v),
            K::Symbol { value: ref v } => write!(f, "{}", v),
            K::Verb { kind: ref v, args: ref a } => {
                if a.len() > 0 {
                    try!(write!(f, "{}", a[0]));
                }
                try!(write!(f, "{}", v));
                for i in 1..a.len() - 1 {
                    try!(write!(f, "{} ", a[i]));
                }
                write!(f, "{}", a[a.len() - 1])
            }
            K::Int { value: v } => write!(f, "{}", v),
            K::Float { value: v } => write!(f, "{}", v),
            K::Lambda { args: ref a, body: ref b } => {
                try!(write!(f, "{{["));
                for i in 0..a.len() - 1 {
                    try!(write!(f, "{};", a[i]));
                }
                try!(write!(f, "{}]", a[a.len() - 1]));
                write!(f, "{}}}", b)
            }
            K::List { values: ref v } => {
                try!(write!(f, "("));
                for i in 0..v.len() - 1 {
                    try!(write!(f, "{};", v[i]));
                }
                write!(f, "{})", v[v.len() - 1])
            }
            K::Dict { keys: ref k, values: ref v } => {
                try!(write!(f, "["));
                for (key, val) in k.iter().zip(v.iter()) {
                    try!(write!(f, "{}:{};", key, val))
                }
                write!(f, "]")
            }
            // Nameref { name: String, value: Box<K> },
            // Adverb {
            //     adverb: Adverb,
            //     left: Box<K>,
            //     verb: Box<K>,
            //     right: Box<K>,
            // },
            _ => write!(f, "nyi"),
        }
    }
}

pub fn verb(s: &str, args: Vec<K>) -> K {
    K::Verb {
        kind: s.to_string(),
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