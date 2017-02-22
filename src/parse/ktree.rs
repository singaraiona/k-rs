use std::mem;
use std::ops::Index;
use std::slice::Iter;
use parse::alloc::Arena;
use parse::arena::ArenaMem;
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
    Verb { kind: u8, args: Vector<K, Id> },
    Ioverb { fd: u8 },
    Int { value: i64 },
    Float { value: f64 },
    Lambda { args: Args, body: Id },
    List { curry: bool, values: Vector<K, Id> },
    Dict {
        keys: Vector<K, Id>,
        values: Vector<K, Id>,
    },
    Nameref { id: u16, value: Id },
    Adverb {
        kind: [u8; 2],
        left: Id,
        verb: Id,
        right: Id,
    },
    Condition { list: Vector<K, Id> },
    Nil,
}

impl K {
    pub fn find_names(&self, arena: &ArenaMem<K, Id>, v: &mut Vec<u16>) -> usize {
        match *self {
            K::Name { value: n } => {
                v.push(n);
                1
            }
            K::Verb { kind: _, args: ref x } => {
                x.iter(&arena).fold(0, |a, ref i| a + i.find_names(arena, v))
            }
            K::Condition { list: ref x } => {
                x.iter(&arena).fold(0, |a, ref i| a + i.find_names(arena, v))
            }
            K::List { curry: _, values: ref x } => {
                x.iter(&arena).fold(0, |a, ref i| a + i.find_names(arena, v))
            }
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
        // K::Verb { kind: ref v, args: ref a } => {
        //     if a.len() > 0 {
        //         pp(&a[0], arena);
        //     }
        //     let _ = write!(f, "{}", *v as char);
        //     for i in 1..a.len() - 1 {
        //         pp(&a[i], arena);
        //     }
        //     pp(&a[a.len() - 1], arena);
        // }
        //
        // K::Lambda { args: ref a, body: ref b } => {
        //     let _ = write!(f, "{{");
        //     a.pp(arena);
        //     pp(b, arena);
        //     let _ = write!(f, "}}");
        // }
        K::List { curry: ref c, values: ref v } => {
            if !c {
                let _ = write!(f, "(");
                for i in 0..v.len() - 1 {
                    pp(v.get(i, &arena.ktree), arena);
                    let _ = write!(f, ";");
                }
                pp(v.get(v.len() - 1, &arena.ktree), arena);
                let _ = write!(f, ")");
            } else {
                for i in 0..v.len() - 1 {
                    pp(v.get(i, &arena.ktree), arena);
                    let _ = write!(f, " ");
                }
                pp(v.get(v.len() - 1, &arena.ktree), arena);
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

pub fn verb(arena: &mut ArenaMem<K, Id>, c: char, args: Vec<K>) -> K {
    K::Verb {
        kind: c as u8,
        args: vector(arena, args),
    }
}

pub fn adverb(arena: &mut ArenaMem<K, Id>, s: String, left: K, verb: K, right: K) -> K {
    let b = s.into_bytes();
    K::Adverb {
        kind: [b[0], b[1]],
        left: atom(arena, left),
        verb: atom(arena, verb),
        right: atom(arena, right),
    }
}

pub fn vector(arena: &mut ArenaMem<K, Id>, mut v: Vec<K>) -> Vector<K, Id> {
    let vec = arena.alloc_vec::<K>(v.len());
    for u in vec.as_slice_mut(arena) {
        *u = v.remove(0);
    }
    vec
}

pub fn list(curry: bool, arena: &mut ArenaMem<K, Id>, mut v: Vec<K>) -> K {
    let vec = arena.alloc_vec::<K>(v.len());
    for u in vec.as_slice_mut(arena) {
        *u = v.remove(0);
    }
    K::List {
        curry: curry,
        values: vec,
    }
}

pub fn dict(arena: &mut ArenaMem<K, Id>, mut keys: Vec<K>, mut values: Vec<K>) -> K {
    let kvec = arena.alloc_vec::<K>(keys.len());
    for u in kvec.as_slice_mut(arena) {
        *u = keys.remove(0);
    }
    let vvec = arena.alloc_vec::<K>(keys.len());
    for u in vvec.as_slice_mut(arena) {
        *u = values.remove(0);
    }
    K::Dict {
        keys: kvec,
        values: vvec,
    }
}

pub fn atom(arena: &mut ArenaMem<K, Id>, k: K) -> Id {
    arena.push(k)
}

pub type Id = u64;