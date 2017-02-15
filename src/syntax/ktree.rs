use std::fmt::{self, Display};
use syntax::error::Error;
use std::mem;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Monad {
    Flip,
    Negate,
    First,
    Sqrt,
    EnumKey,
    Where,
    Reverse,
    Asc,
    Desc,
    Group,
    Not,
    Enlist,
    Null,
    Count,
    Floor,
    STring,
    Distinct,
    Type,
    Eval,
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Dyad {
    Plus,
    Minus,
    Times,
    Divide,
    ModMap,
    MinAnd,
    MaxOr,
    Less,
    More,
    Equal,
    Match,
    Concat,
    Except,
    TakeRsh,
    DropCut,
    CastSum,
    FindRnd,
    At,
    Dot,
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Appliance {
    Monadic,
    Dyadic,
}

#[derive(Debug, Clone)]
pub enum Verb {
    Monad(Monad),
    Dyad(Dyad),
    IoVerb(u8),
    Internal(String),
}

impl Verb {
    pub fn construct(s: &str, a: Appliance) -> Result<Self, Error> {
        lazy_static! {
            static ref VERBS: Vec<String> = "+\x2D*%!&|<>=~,^#_$?@."
            .chars().map(|x| x.to_string()).collect();
        }
        match VERBS.iter().position(|ref x| *x == s) {
            Some(x) => unsafe {
                match a {                    
                    Appliance::Monadic => Ok(Verb::Monad(mem::transmute(x as u8))),
                    Appliance::Dyadic => Ok(Verb::Dyad(mem::transmute(x as u8))),
                }
            },
            None => Err(Error::ParseError(format!("Can not construct Verb from {}.", &s))),
        }
    }
}

impl Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _ => write!(f, "+"),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Adverb {
    Each,
    OverJoin,
    ScanSplit,
    Eachprior,
    Eachright,
    Eachleft,
}

impl Adverb {
    pub fn construct(s: &str) -> Result<Self, Error> {
        lazy_static! {
            static ref ADVERBS: Vec<String> = ["'", "/", "\\", "':", "\\:", "/:"]
            .iter().map(|x| x.to_string()).collect();
        }
        match ADVERBS.iter().position(|ref x| *x == s) {
            Some(x) => unsafe { Ok(mem::transmute(x as u8)) },
            None => Err(Error::ParseError(format!("Can not construct Adverb from {}.", &s))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum K {
    Name { value: String },
    Symbol { value: String },
    Verb { verb: Verb, args: Vec<K> },
    Int { value: i64 },
    Float { value: f64 },
    Lambda { args: Vec<String>, body: Box<K> },
    List { values: Vec<K> },
    Dict { keys: Vec<K>, values: Vec<K> },
    Nameref { name: String, value: Box<K> },
    Adverb {
        adverb: Adverb,
        left: Box<K>,
        verb: Box<K>,
        right: Box<K>,
    },
    Condition { list: Vec<K> },
    Nil,
}

impl K {
    pub fn as_u8(&self) -> u8 {
        match *self {
            K::Name { .. } => 0,
            K::Symbol { .. } => 1,
            K::Verb { .. } => 2,
            K::Int { .. } => 3,
            K::Float { .. } => 4,
            K::Lambda { .. } => 5,
            K::List { .. } => 6,
            K::Dict { .. } => 7,
            K::Nameref { .. } => 8,
            K::Adverb { .. } => 9,
            K::Condition { .. } => 10,
            K::Nil => 13,
        }
    }

    pub fn find_names(&self, v: &mut Vec<String>) -> usize {
        match *self {
            K::Name { value: ref n } => {
                v.push(n.clone());
                1
            }
            K::Verb { verb: _, args: ref x } => x.iter().fold(0, |a, ref i| a + i.find_names(v)),
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
            K::Verb { verb: ref v, args: ref a } => {
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