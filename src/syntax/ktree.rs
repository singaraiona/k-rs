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
pub enum Applience {
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
    pub fn construct(s: &str, a: Applience) -> Result<Self, Error> {
        lazy_static! {
            static ref VERBS: Vec<String> = "+\x2D*%!&|<>=~,^#_$?@."
            .chars().map(|x| x.to_string()).collect();
        }
        match VERBS.iter().position(|ref x| *x == s) {
            Some(x) => unsafe {
                match a {                    
                    Applience::Monadic => Ok(Verb::Monad(mem::transmute(x as u8))),
                    Applience::Dyadic => Ok(Verb::Dyad(mem::transmute(x as u8))),
                }
            },
            None => Err(Error::ParseError(format!("Can not construct Verb from {}.", &s))),
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
            K::Nil => 13,
        }
    }

    pub fn find_names(&self, v: &mut Vec<String>) {
        match *self {
            K::Name { value: ref n } => v.push(n.clone()),
            K::Verb { verb: _, args: ref x } => {
                for i in x.iter() {
                    i.find_names(v);
                }
            }
            K::Lambda { args: _, body: ref x } => {
                x.find_names(v);
            }
            K::List { values: ref x } => {
                for i in x.iter() {
                    i.find_names(v);
                }
            }
            _ => (),
        }
    }
}

impl PartialEq for K {
    fn eq(&self, other: &K) -> bool {
        self.as_u8() == other.as_u8()
    }
}