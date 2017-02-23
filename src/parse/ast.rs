use std::mem;
use std::fmt;
use std::str;
use std::ops::Index;
use std::slice::Iter;
use parse::alloc::Arena;
use parse::arena::ArenaMem;
use parse::vector::Vector;
use std::io::{stdout, Write};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum Adverb {
    Each,
    OverJoin,
    ScanSplit,
    EachPrior,
    Eachright,
    Eachleft,
}

impl fmt::Display for Adverb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Adverb::Each => write!(f, "{}", "'"),
            Adverb::OverJoin => write!(f, "{}", "/"),
            Adverb::ScanSplit => write!(f, "{}", "\\"),
            Adverb::EachPrior => write!(f, "{}", "':"),
            Adverb::Eachright => write!(f, "{}", "/:"),
            Adverb::Eachleft => write!(f, "{}", "\\:"),
        }
    }
}

impl str::FromStr for Adverb {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s[..] {
            "'" => Ok(Adverb::Each),
            "/" => Ok(Adverb::OverJoin),
            "\\" => Ok(Adverb::ScanSplit),
            "':" => Ok(Adverb::EachPrior),
            "/:" => Ok(Adverb::Eachright),
            "\\:" => Ok(Adverb::Eachleft),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AST {
    Name { value: u16 },
    Bool { value: bool },
    Symbol { value: u16 },
    Verb { kind: u8, args: Vector<AST, Id> },
    Ioverb { fd: u8 },
    Int { value: i64 },
    Float { value: f64 },
    Lambda { args: Args, body: Id },
    List {
        curry: bool,
        values: Vector<AST, Id>,
    },
    Sequence { values: Vector<AST, Id> },
    Dict {
        keys: Vector<AST, Id>,
        values: Vector<AST, Id>,
    },
    Nameref { name: u16, value: Id },
    Adverb {
        kind: Adverb,
        left: Id,
        verb: Id,
        right: Id,
    },
    Condition { list: Vector<AST, Id> },
    Quit,
    Nil,
}

impl AST {
    pub fn find_names(&self, arena: &ArenaMem<AST, Id>, v: &mut Vec<u16>) -> usize {
        match *self {
            AST::Name { value: n } => {
                v.push(n);
                1
            }
            AST::Verb { kind: _, args: ref x } => {
                x.iter(&arena).fold(0, |a, ref i| a + i.find_names(arena, v))
            }
            AST::Condition { list: ref x } => {
                x.iter(&arena).fold(0, |a, ref i| a + i.find_names(arena, v))
            }
            AST::List { curry: _, values: ref x } => {
                x.iter(&arena).fold(0, |a, ref i| a + i.find_names(arena, v))
            }
            _ => 0,
        }
    }

    pub fn is_atom(&self) -> bool {
        match *self {
            AST::Int { .. } => true,
            AST::Float { .. } => true,
            AST::Symbol { .. } => true,
            _ => false,
        }
    }

    pub fn type_eq(&self, other: &AST) -> bool {
        self.is_atom() && other.is_atom()
    }
}

impl PartialEq for AST {
    fn eq(&self, other: &AST) -> bool {
        mem::discriminant(self) == mem::discriminant(&other)
    }
}

pub fn pp(ast: &AST, arena: &Arena) {
    let mut f = stdout();
    match *ast {
        AST::Name { value: v } => {
            let _ = write!(f, "{}", arena.id_name(v));
        }
        AST::Bool { value: v } => {
            let _ = write!(f, "{}b", v as u8);
        }
        AST::Symbol { value: v } => {
            let _ = write!(f, "`{}", arena.id_symbol(v));
        }
        AST::Int { value: v } => {
            let _ = write!(f, "{}", v);
        }
        AST::Float { value: v } => {
            let _ = write!(f, "{}", v);
        }
        AST::Verb { kind: ref v, args: ref a } => {
            let s = &a.as_slice(&arena.ast);
            if s.len() > 0 {
                pp(&s[0], arena);
            }
            let _ = write!(f, "{}", *v as char);
            if s.len() > 1 {
                for i in 1..s.len() - 1 {
                    pp(&s[i], arena);
                }
                pp(&s[s.len() - 1], arena);
            }
        }
        AST::Lambda { args: ref a, body: ref b } => {
            let _ = write!(f, "{{");
            a.pp(arena);
            let u = arena.ast.deref(*b);
            pp(u, arena);
            let _ = write!(f, "}}");
        }
        AST::List { curry: ref c, values: ref v } => {
            if v.len() == 0 {
                let _ = write!(f, "()");
            } else if v.len() == 1 {
                let _ = write!(f, ",");
                pp(v.get(0, &arena.ast), arena);
            } else {
                if unified(&arena.ast, v) {
                    for i in 0..v.len() - 1 {
                        pp(v.get(i, &arena.ast), arena);
                        let _ = write!(f, " ");
                    }
                    pp(v.get(v.len() - 1, &arena.ast), arena);
                } else {
                    let _ = write!(f, "(");
                    for i in 0..v.len() - 1 {
                        pp(v.get(i, &arena.ast), arena);
                        let _ = write!(f, ";");
                    }
                    pp(v.get(v.len() - 1, &arena.ast), arena);
                    let _ = write!(f, ")");
                }
            }
        }        
        AST::Dict { keys: ref k, values: ref v } => {
            let _ = write!(f, "[");
            let u = k.as_slice(&arena.ast);
            let m = v.as_slice(&arena.ast);
            for (key, val) in u[..u.len() - 1].iter().zip(m) {
                pp(key, arena);
                let _ = write!(f, ":");
                pp(val, arena);
                let _ = write!(f, ";");
            }
            pp(&u[u.len() - 1], arena);
            let _ = write!(f, ":");
            pp(&m[m.len() - 1], arena);
            let _ = write!(f, "]");
        }
        AST::Condition { list: ref c } => {
            let l = c.as_slice(&arena.ast);
            let _ = write!(f, "$[");
            for i in 0..l.len() - 1 {
                pp(&l[i], arena);
                let _ = write!(f, ";");
            }
            pp(&l[l.len() - 1], arena);
            let _ = write!(f, "]");
        }
        AST::Adverb { kind: ref k, left: l, verb: v, right: r } => {
            pp(&arena.ast.deref(l), arena);
            pp(&arena.ast.deref(v), arena);
            let _ = write!(f, "{}", k);
            pp(&arena.ast.deref(r), arena);
        }
        AST::Nil => (),
        _ => {
            let _ = write!(f, "nyi");
        }
    };
}

pub fn verb(arena: &mut ArenaMem<AST, Id>, c: char, args: Vec<AST>) -> AST {
    AST::Verb {
        kind: c as u8,
        args: vector(arena, args),
    }
}

pub fn adverb(arena: &mut ArenaMem<AST, Id>, s: String, left: AST, verb: AST, right: AST) -> AST {
    // let b = s.into_bytes();
    let kind = s.parse::<Adverb>().expect("Invalid adverb.");

    AST::Adverb {
        kind: kind,
        left: atom(arena, left),
        verb: atom(arena, verb),
        right: atom(arena, right),
    }
}

pub fn vector(arena: &mut ArenaMem<AST, Id>, mut v: Vec<AST>) -> Vector<AST, Id> {
    let vec = arena.alloc_vec::<AST>(v.len());
    for u in vec.as_slice_mut(arena) {
        *u = v.remove(0);
    }
    vec
}

pub fn list(curry: bool, arena: &mut ArenaMem<AST, Id>, mut v: Vec<AST>) -> AST {
    let vec = arena.alloc_vec::<AST>(v.len());
    for u in vec.as_slice_mut(arena) {
        *u = v.remove(0);
    }
    AST::List {
        curry: curry,
        values: vec,
    }
}

pub fn sequence(arena: &mut ArenaMem<AST, Id>, mut v: Vec<AST>) -> AST {
    let vec = arena.alloc_vec::<AST>(v.len());
    for u in vec.as_slice_mut(arena) {
        *u = v.remove(0);
    }
    AST::Sequence { values: vec }
}

pub fn dict(arena: &mut ArenaMem<AST, Id>, mut keys: Vec<AST>, mut values: Vec<AST>) -> AST {
    let kvec = arena.alloc_vec::<AST>(keys.len());
    for u in kvec.as_slice_mut(arena) {
        *u = keys.remove(0);
    }
    let vvec = arena.alloc_vec::<AST>(keys.len());
    for u in vvec.as_slice_mut(arena) {
        *u = values.remove(0);
    }
    AST::Dict {
        keys: kvec,
        values: vvec,
    }
}

pub fn atom(arena: &mut ArenaMem<AST, Id>, ast: AST) -> Id {
    arena.push(ast)
}

pub fn unified(arena: &ArenaMem<AST, Id>, vec: &Vector<AST, Id>) -> bool {
    let mut it = vec.iter(arena);
    let o = it.next();
    if let Some(t) = o {
        for i in it {
            if !i.type_eq(t) {
                return false;
            }
        }
        return true;
    }
    false
}

pub type Id = u64;