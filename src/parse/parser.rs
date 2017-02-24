use std::str;
use parse::error::Error;
use parse::ast::{self, AST, Args};
use parse::token::{Token, Raw};
use regex::Regex;
use parse::alloc::Arena;

pub struct Parser {
    text: String,
}

macro_rules! extract {
    ($k:expr) => (match $k {
        AST::Int{value: v} => v,
        _ => unimplemented!(),
    })
}

impl Parser {
    fn begin(&mut self, s: &str) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(\x22(?:[^\x22\x5C\n]|\.)*\x22)|[a-zA-Z]*[ ]+(/.*)|([a-z\d\]\)]-\.?\d+)|.")
                .unwrap();
        }
        // preserve a string, remove a comment, disambiguate a minus sign.
        self.text = RE.captures_iter(s.trim())
            .fold("".to_string(), |acc, cap| {
                if cap.get(1).is_some() {
                    acc + &cap[1]
                } else if cap.get(2).is_some() {
                    acc + &str::replace(&cap[0], &cap[2], "")[..]
                } else if cap.get(3).is_some() {
                    acc + &str::replace(&cap[3], "-", "- ")[..]
                } else {
                    match acc.chars().rev().nth(0) {
                        Some(c) => {
                            if c.is_digit(10) {
                                acc + &str::replace(&cap[0], "-", "- ")[..]
                            } else {
                                acc + &cap[0]
                            }
                        }
                        _ => acc + &cap[0],
                    }
                }
            });
        self.text = self.text.replace("\n", ";");
    }

    pub fn parse(&mut self, b: &[u8], arena: &mut Arena) -> Result<AST, Error> {
        self.begin(str::from_utf8(b).expect("Invalid input."));
        self.parse_list(arena, None)
    }

    pub fn parse_str(&mut self, s: &str, arena: &mut Arena) -> Result<AST, Error> {
        self.begin(s);
        self.parse_list(arena, None)
    }

    #[inline]
    fn expect(&mut self, t: Token) -> Result<Raw, Error> {
        match t.find(&self.text) {
            Some(x) => {
                self.text = self.text[x.len()..].trim_left().to_string();
                Ok(x)
            }
            None => Err(Error::ParseError(format!("Expected: {:?}", t))),
        }
    }

    #[inline]
    fn at(&self, t: Token) -> bool {
        t.is_match(&self.text)
    }

    #[inline]
    fn matches(&mut self, t: Token) -> Option<Raw> {
        if self.at(t) {
            return match self.expect(t) {
                Ok(x) => Some(x),
                Err(..) => None,
            };
        }
        None
    }

    #[inline]
    fn done(&self) -> bool {
        self.text.is_empty()
    }

    #[inline]
    fn at_noun(&self) -> bool {
        !self.done() && self.at(Token::Number) || self.at(Token::Name) ||
        self.at(Token::Symbol) || self.at(Token::String) || self.at(Token::Cond) ||
        self.at(Token::OpenP) || self.at(Token::OpenC)
    }

    fn applycallright(&mut self, arena: &mut Arena, node: AST) -> Result<AST, Error> {
        let mut ret = node;
        while self.matches(Token::OpenB).is_some() {
            let args = try!(self.parse_list(arena, Some(Token::CloseB)));
            ret = ast::verb(&mut arena.ast, '.', vec![ret, args]);
        }
        Ok(ret)
    }

    fn applyindexright(&mut self, arena: &mut Arena, node: AST) -> Result<AST, Error> {
        // if (node.sticky && at(VERB)) {
        //     if self.at(Token::Verb) {
        //  let x = try!(self.parseNoun());
        //     x.l = node;
        //     let r = try!(self.parse_ex(parseNoun());
        //     return x;
        // }
        let mut r = node;
        while self.matches(Token::OpenB).is_some() {
            let e = try!(self.parse_list(arena, Some(Token::CloseB)));
            r = ast::verb(&mut arena.ast, '.', vec![r, e]);
        }
        return Ok(r);
    }

    fn parse_adverb(&mut self, arena: &mut Arena, left: AST, verb: AST) -> Result<AST, Error> {
        let a = try!(self.expect(Token::Adverb));
        // while self.at(Token::Adverb) {
        //     let b = try!(self.expect(Token::Adverb));
        //     // here will be parsing adverb from Raw ..
        //     verb = AST::Verb {
        //         kind: verb,
        //         args: vec![verb],
        //     };
        //     a = b;
        // }
        // if (at(OPEN_B)) { return applycallright({ t:9, v:a, kind:verb, l:left }); }
        let n = try!(self.parse_noun(arena));
        let right = try!(self.parse_ex(arena, n));
        return Ok(ast::adverb(&mut arena.ast, a.value().to_string(), left, verb, right));
    }

    #[inline]
    fn parse_noun(&mut self, arena: &mut Arena) -> Result<AST, Error> {
        if self.matches(Token::Quit).is_some() {
            return Ok(AST::Quit);
        }
        if self.matches(Token::Colon).is_some() {
            let b = arena.intern_name("y".to_string());
            return Ok(AST::Lambda {
                args: args![arena.intern_name_id("x".to_string()),
                            arena.intern_name_id("y".to_string())],
                body: ast::atom(&mut arena.ast, b),
            });
        }
        if self.at(Token::Ioverb) {
            let v = try!(self.expect(Token::Ioverb));
            return match v.value()[..1].parse::<u8>() {
                Ok(x) => Ok(AST::Ioverb { fd: x }),
                Err(_) => Err(Error::UnexpectedToken),
            };
        }
        if self.at(Token::Bool) {
            let n = try!(self.expect(Token::Bool));
            let v: Vec<AST> = n.value()[..n.value().len() - 1]
                .chars()
                .map(|x| AST::Int { value: (x as i64) - 0x30 })
                .collect();
            let list = ast::list(true, &mut arena.ast, v);
            return self.applyindexright(arena, list);
        }
        if self.at(Token::Hexlit) {
            let h = try!(self.expect(Token::Hexlit));
            return match i64::from_str_radix(&h.value()[2..], 16) {
                Ok(v) => Ok(AST::Int { value: v }),
                Err(_) => Err(Error::ParseError(format!("Malformed byte string."))),
            };
        }
        if self.matches(Token::Cond).is_some() {
            return match try!(self.parse_list(arena, Some(Token::CloseB))) {
                AST::Sequence { values: v } => Ok(AST::Condition { list: v }),
                _ => Err(Error::InvalidCondition),
            };
        }
        if self.at(Token::Number) {
            let mut v: Vec<AST> = Vec::new();
            while self.at(Token::Number) {
                let mut n = try!(self.expect(Token::Number));
                n = n.trim_right_matches(|c| c == 'i' || c == 'f');
                match n.parse::<i64>() {
                    Ok(x) => v.push(AST::Int { value: x }),
                    Err(_) => {
                        let x = try!(n.parse::<f64>());
                        v.push(AST::Float { value: x });
                    }
                }
            }
            return match v.len() {
                1 => self.applyindexright(arena, v.pop().unwrap()),
                _ => {
                    let list = ast::list(true, &mut arena.ast, v);
                    self.applyindexright(arena, list)
                }
            };
        }
        if self.at(Token::Verb) {
            let n = try!(self.expect(Token::Verb));
            // here is unclear point,
            // for now it's just creates Monadic verb.
            let v = match self.matches(Token::Colon) {
                Some(..) => n.value(),
                None => n.value(),
            };
            //
            if self.at(Token::OpenB) && !self.at(Token::Dict) {
                let _ = try!(self.expect(Token::OpenB));
                let r = try!(self.parse_list(arena, Some(Token::CloseB)));
                return Ok(ast::verb(&mut arena.ast,
                                    v.chars().nth(0).expect("Char expected."),
                                    vec![r]));
            }
            return Ok(ast::verb(&mut arena.ast,
                                v.chars().nth(0).expect("Char expected."),
                                vec![]));
        }
        if self.at(Token::Symbol) {
            let mut v: Vec<AST> = Vec::new();
            while self.at(Token::Symbol) {
                let n = try!(self.expect(Token::Symbol));
                let mut t = try!(n.parse::<String>());
                t.remove(0);
                v.push(arena.intern_symbol(t));
            }
            return match v.len() {
                1 => self.applyindexright(arena, v.pop().unwrap()),
                _ => {
                    let list = ast::list(true, &mut arena.ast, v);
                    self.applyindexright(arena, list)
                }
            };
        }
        if self.at(Token::String) {
            let s = try!(self.expect(Token::String));
            let mut t = try!(s.parse::<String>());
            t.remove(0);
            t.pop();
            if let Some(v) = ast::Chars::new(&mut t) {
                return self.applyindexright(arena, AST::String { value: v });
            }
            return Err(Error::StringSize);
        }
        if self.at(Token::Name) {
            let n = try!(self.expect(Token::Name));
            let t = try!(n.parse::<String>());
            if let Some(x) = arena.native_id(&t) {
                return self.applycallright(arena, AST::Native { name: x });
            }
            if self.matches(Token::Colon).is_some() {
                let _ = self.matches(Token::Colon).is_some();
                let n = try!(self.parse_noun(arena));
                let r = try!(self.parse_ex(arena, n));
                if r == AST::Nil {
                    return Err(Error::ParseError(format!("Noun expected following ':'.")));
                }
                return Ok(AST::Nameref {
                    name: arena.intern_name_id(t),
                    value: ast::atom(&mut arena.ast, r),
                });
            }
            if self.matches(Token::OpenB).is_some() {
                let index = try!(self.parse_list(arena, Some(Token::CloseB)));
                // if (at(ASSIGN)) { return compoundassign(n, index); }
                // if (matches(COLON)) { return indexedassign(n, index); }
                // if (index.length == 0) { index = [NIL]; }
                let u = arena.intern_name(t);
                return Ok(ast::verb(&mut arena.ast, '.', vec![u, index]));
            }
            return Ok(arena.intern_name(t));
        }
        if self.matches(Token::OpenB).is_some() {
            let mut keys: Vec<AST> = Vec::new();
            let mut values: Vec<AST> = Vec::new();
            if self.matches(Token::CloseB).is_none() {
                loop {
                    let key = try!(self.expect(Token::Name));
                    let _ = self.expect(Token::Colon);
                    let n = try!(self.parse_noun(arena));
                    let value = try!(self.parse_ex(arena, n));
                    let kname = arena.intern_name(try!(key.parse::<String>()));
                    keys.push(kname);
                    values.push(value);
                    if self.matches(Token::Semi).is_none() {
                        break;
                    }
                }
                let _ = try!(self.expect(Token::CloseB));
            }
            return Ok(ast::dict(&mut arena.ast, keys, values));
        }
        if self.matches(Token::OpenC).is_some() {
            let mut args = Args::new();
            if self.matches(Token::OpenB).is_some() {
                loop {
                    if self.at(Token::CloseB) {
                        break;
                    }
                    let n = try!(self.expect(Token::Name));
                    let t = try!(n.parse::<String>());
                    args.push(arena.intern_name_id(t));
                    if self.matches(Token::Semi).is_none() {
                        break;
                    }
                }
                let _ = try!(self.expect(Token::CloseB));
            }
            let r = try!(self.parse_list(arena, Some(Token::CloseC)));
            if args.len() == 0 {
                let mut names: Vec<u16> = Vec::new();
                r.find_names(&arena.ast, &mut names);
                if names.contains(&arena.name_id("z")) {
                    args.push(arena.intern_name_id(String::from("x")));
                    args.push(arena.intern_name_id(String::from("y")));
                    args.push(arena.intern_name_id(String::from("z")));
                } else if names.contains(&arena.name_id("y")) {
                    args.push(arena.intern_name_id(String::from("x")));
                    args.push(arena.intern_name_id(String::from("y")));
                } else if names.contains(&arena.name_id("x")) {
                    args.push(arena.intern_name_id(String::from("x")));
                }
            }
            let b = ast::atom(&mut arena.ast, r);
            return self.applycallright(arena,
                                       AST::Lambda {
                                           args: args,
                                           body: b,
                                       });
        }
        if self.matches(Token::OpenP).is_some() {
            let n = try!(self.parse_list(arena, Some(Token::CloseP)));
            let r = match n {
                AST::Sequence { values: v } => {
                    AST::List {
                        curry: false,
                        values: v,
                    }
                }
                x => x,
            };
            return self.applyindexright(arena, r);
        }
        Ok(AST::Nil)
    }

    fn parse_ex(&mut self, arena: &mut Arena, node: AST) -> Result<AST, Error> {
        if node == AST::Nil {
            return Ok(AST::Nil);
        }
        if self.at(Token::Adverb) {
            return self.parse_adverb(arena, AST::Nil, node);
        }
        if self.at_noun() && !self.at(Token::Ioverb) {
            let n = try!(self.parse_noun(arena));
            if self.at(Token::Adverb) {
                return self.parse_adverb(arena, node, n);
            }
            let p = try!(self.parse_ex(arena, n));
            return match node {
                AST::Verb { kind: v, args: a } => {
                    if a.len() == 0 {
                        Ok(ast::verb(&mut arena.ast, v as char, vec![p]))
                    } else {
                        Ok(AST::Verb {
                            kind: '@' as u8,
                            args: a,
                        })
                    }
                }
                x => Ok(ast::verb(&mut arena.ast, '@', vec![x, p])),
            };
        }
        if self.at(Token::Verb) {
            let n = try!(self.expect(Token::Verb));
            let v = n.value();
            if self.at(Token::Adverb) {
                let u = ast::verb(&mut arena.ast,
                                  v.chars().nth(0).expect("Char expected."),
                                  vec![]);
                return self.parse_adverb(arena, node, u);
            }
            let x = try!(self.parse_noun(arena));
            let r = try!(self.parse_ex(arena, x));
            return Ok(ast::verb(&mut arena.ast,
                                v.chars().nth(0).expect("Char expected."),
                                vec![node, r]));
        }
        Ok(node)
    }

    fn parse_list(&mut self, arena: &mut Arena, terminal: Option<Token>) -> Result<AST, Error> {
        let mut vec: Vec<AST> = Vec::new();
        loop {
            if terminal.is_some() && self.at(terminal.unwrap()) {
                break;
            }
            let n = try!(self.parse_noun(arena));
            match self.parse_ex(arena, n) {
                Ok(a) => vec.push(a),
                Err(e) => return Err(e),
            }
            if self.matches(Token::Semi).is_none() {
                break;
            }
        }
        if let Some(x) = terminal {
            let _ = try!(self.expect(x));
        };
        match vec.len() {
            0 => Ok(AST::Nil),
            1 => Ok(vec.pop().unwrap()),
            _ => Ok(ast::sequence(&mut arena.ast, vec)),
        }
    }
}

pub fn new() -> Parser {
    Parser { text: String::new() }
}
