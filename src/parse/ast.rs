use std::mem;
use std::fmt;
use std::str;
use std::ops::Index;
use std::slice::Iter;
use parse::alloc::Arena;
use parse::arena::ArenaMem;
use parse::vector::Vector;
use handle;

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
}

impl<'a> fmt::Display for Land<'a, Args> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (args, arena) = (self.0, self.1);
        let a = args.args;
        let l = args.len();
        try!(write!(f, "["));
        for i in 0..l - 1 {
            try!(write!(f, "{};", arena.id_name(a[i])));
        }
        write!(f, "{}]", arena.id_name(a[l - 1]))
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

// For now it's just u8. Should be coerced to unicode in the future.
pub struct Chars {
    raw: [u8; 64],
    len: u8,
}

impl Chars {
    pub fn new(r: &str) -> Option<Self> {
        if r.len() > 64 {
            return None;
        }
        let mut raw = [0u8; 64];
        let b = r.as_bytes();
        for i in 0..b.len() {
            raw[i] = b[i];
        }
        Some(Chars {
            raw: raw,
            len: r.len() as u8,
        })
    }

    pub fn to_string(&self) -> String {
        match str::from_utf8(&self.raw[..]) {
            Ok(s) => s.to_string(),
            Err(..) => "".to_string(),            
        }
    }
}

impl Copy for Chars {}

impl Clone for Chars {
    fn clone(&self) -> Self {
        *self
    }
}

impl fmt::Debug for Chars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chars {:?}", &self.raw[..self.len as usize])
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AST {
    Bool { value: bool },
    Name { value: u16 },
    Symbol { value: u16 },
    String { value: Chars },
    Verb { kind: u8, args: Vector<AST, Id> },
    Ioverb { fd: u8 },
    Int { value: i64 },
    Float { value: f64 },
    Lambda { args: Args, body: Id },
    Native { name: u16 },
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
    Debug { value: Id },
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

    pub fn type_id(&self) -> i8 {
        match *self {
            AST::Int { .. } => -7,
            AST::Float { .. } => -8,
            AST::Symbol { .. } => -9,
            _ => !0 as i8,
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

impl<'a> fmt::Display for Land<'a, AST> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ast, arena) = (self.0, self.1);
        match *ast {
            AST::Name { value: v } => write!(f, "{}", arena.id_name(v)),
            AST::Bool { value: v } => write!(f, "{}b", v as u8),
            AST::Symbol { value: v } => write!(f, "`{}", arena.id_symbol(v)),
            AST::String { value: ref s } => write!(f, "\"{}\"", s.to_string()),
            AST::Int { value: v } => write!(f, "{}", v),
            AST::Float { value: v } => write!(f, "{}", v),
            AST::Verb { kind: ref v, args: ref a } => {
                let s = &a.as_slice(&arena.ast);
                if s.len() > 0 {
                    try!(write!(f, "{}", Land(&s[0], arena)));
                }
                try!(write!(f, "{}", *v as char));
                if s.len() > 1 {
                    for i in 1..s.len() - 1 {
                        try!(write!(f, "{}", Land(&s[i], arena)));
                    }
                    try!(write!(f, "{}", Land(&s[s.len() - 1], arena)));
                }
                Ok(())
            }
            AST::Lambda { args: ref a, body: ref b } => {
                try!(write!(f, "{{{}", Land(a, arena)));
                let u = arena.ast.deref(*b);
                write!(f, "{}}}", Land(u, arena))
            }
            AST::List { curry: _, values: ref v } => {
                if v.len() == 0 {
                    Ok(())
                } else if v.len() == 1 {
                    write!(f, ",{}", Land(v.get(0, &arena.ast), arena))
                } else {
                    if is_unified(&arena.ast, v) {
                        for i in 0..v.len() - 1 {
                            try!(write!(f, "{} ", Land(v.get(i, &arena.ast), arena)));
                        }
                        write!(f, "{}", Land(v.get(v.len() - 1, &arena.ast), arena))
                    } else {
                        if is_flat(&arena.ast, v) {
                            try!(write!(f, "("));
                            for i in 0..v.len() - 1 {
                                try!(write!(f, "{};", Land(v.get(i, &arena.ast), arena)));
                            }
                            write!(f, "{})", Land(v.get(v.len() - 1, &arena.ast), arena))
                        } else {
                            for i in 0..v.len() - 1 {
                                try!(write!(f, "{}\n", Land(v.get(i, &arena.ast), arena)));
                            }
                            write!(f, "{}", Land(v.get(v.len() - 1, &arena.ast), arena))
                        }
                    }
                }
            }
            AST::Dict { keys: ref k, values: ref v } => {
                try!(write!(f, "["));
                let u = k.as_slice(&arena.ast);
                let m = v.as_slice(&arena.ast);
                for (key, val) in u[..u.len() - 1].iter().zip(m) {
                    try!(write!(f, "{}:{};", Land(key, arena), Land(val, arena)));
                }
                write!(f,
                       "{}:{}]",
                       Land(&u[u.len() - 1], arena),
                       Land(&m[m.len() - 1], arena))
            }
            AST::Condition { list: ref c } => {
                let l = c.as_slice(&arena.ast);
                let _ = write!(f, "$[");
                for i in 0..l.len() - 1 {
                    try!(write!(f, "{};", Land(&l[i], arena)));
                }
                write!(f, "{}]", Land(&l[l.len() - 1], arena))
            }
            AST::Adverb { kind: ref k, left: l, verb: v, right: r } => {
                write!(f,
                       "{}{}{}{}",
                       Land(arena.ast.deref(l), arena),
                       Land(arena.ast.deref(v), arena),
                       k,
                       Land(arena.ast.deref(r), arena))
            }
            AST::Native { name: n } => write!(f, "-{}!", n),
            AST::Debug { value: n } => write!(f, "{:#?}", arena.ast.deref(n)),
            AST::Nil => Ok(()),
            _ => write!(f, "nyi"),
        }
    }
}

pub fn print(ast: &AST, arena: &Arena) {
    match ast {
        &AST::Nil => (),
        a => println!("{}", Land(a, arena)),
    }
}

pub fn verb(arena: &mut ArenaMem<AST, Id>, c: char, args: Vec<AST>) -> AST {
    AST::Verb {
        kind: c as u8,
        args: vector(arena, args),
    }
}

pub fn adverb(arena: &mut ArenaMem<AST, Id>, s: String, left: AST, verb: AST, right: AST) -> AST {
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

pub fn concat(arena: &mut ArenaMem<AST, Id>,
              left: &Vector<AST, Id>,
              right: &Vector<AST, Id>)
              -> AST {

    let vec = arena.alloc_vec::<AST>(left.len() + right.len());
    let h = handle::into_raw(arena);
    let li = left.iter(handle::from_raw(h));
    let ri = right.iter(handle::from_raw(h));
    let it = li.chain(ri);
    for (u, v) in vec.as_slice_mut(handle::from_raw(h)).iter_mut().zip(it) {
        *u = *v;
    }
    AST::List {
        curry: true,
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
    let vvec = arena.alloc_vec::<AST>(values.len());
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

pub fn is_unified(arena: &ArenaMem<AST, Id>, vec: &Vector<AST, Id>) -> bool {
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

pub fn is_flat(arena: &ArenaMem<AST, Id>, vec: &Vector<AST, Id>) -> bool {
    for x in vec.iter(arena) {
        if !x.is_atom() {
            return false;
        }
    }
    true
}

struct Land<'a, T: 'a>(&'a T, &'a Arena);

pub type Id = u64;