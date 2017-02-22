use std::str;
use parse::error::Error;
use parse::ktree::{self, K, Args};
use parse::token::{Token, Raw};
use regex::Regex;
use parse::alloc::Arena;

pub struct Parser {
    text: String,
    natives: Vec<(String, K)>,
}

macro_rules! extract {
    ($k:expr) => (match $k {
        K::Int{value: v} => v,
        _ => unimplemented!(),
    })
}

impl Parser {
    fn begin(&mut self, s: &str) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(\x22(?:[^\x22\x5C\n]|\.)*\x22)|[a-zA-Z]*(/.*)|([a-z\d\]\)]-\.?\d+)|.")
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

    pub fn parse(&mut self, b: &[u8], arena: &mut Arena) -> Result<K, Error> {
        self.begin(str::from_utf8(b).expect("Invalid input."));
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
        self.at(Token::Symbol) || self.at(Token::Char) || self.at(Token::Cond) ||
        self.at(Token::OpenP) || self.at(Token::OpenC)
    }

    fn applycallright(&mut self, arena: &mut Arena, node: K) -> Result<K, Error> {
        let mut ret = node;
        while self.matches(Token::OpenB).is_some() {
            let args = try!(self.parse_list(arena, Some(Token::CloseB)));
            ret = ktree::verb('.', vec![ret, args]);
        }
        Ok(ret)
    }

    fn applyindexright(&mut self, arena: &mut Arena, node: K) -> Result<K, Error> {
        // if (node.sticky && at(VERB)) {
        //     if self.at(Token::Verb) {
        // 	let x = try!(self.parseNoun());
        //     x.l = node;
        //     let r = try!(self.parse_ex(parseNoun());
        //     return x;
        // }
        let mut r = node;
        while self.matches(Token::OpenB).is_some() {
            let e = try!(self.parse_list(arena, Some(Token::CloseB)));
            r = ktree::verb('.', vec![r, e]);
        }
        return Ok(r);
    }

    fn parse_adverb(&mut self, arena: &mut Arena, left: K, verb: K) -> Result<K, Error> {
        let a = try!(self.expect(Token::Adverb));
        // while self.at(Token::Adverb) {
        //     let b = try!(self.expect(Token::Adverb));
        //     // here will be parsing adverb from Raw ..
        //     verb = K::Verb {
        //         kind: verb,
        //         args: vec![verb],
        //     };
        //     a = b;
        // }
        // if (at(OPEN_B)) { return applycallright({ t:9, v:a, kind:verb, l:left }); }
        let n = try!(self.parse_noun(arena));
        let right = try!(self.parse_ex(arena, n));
        return Ok(ktree::adverb(a.value().to_string(), box left, box verb, box right));
    }

    #[inline]
    fn native(&self, s: &String) -> Option<K> {
        self.natives.iter().find(|&x| *x.0 == *s).map(|ref x| x.1.clone())
    }

    #[inline]
    fn parse_noun(&mut self, arena: &mut Arena) -> Result<K, Error> {
        if self.matches(Token::Colon).is_some() {
            return Ok(K::Lambda {
                args: args![arena.intern_name_id("x".to_string()),
                            arena.intern_name_id("y".to_string())],
                body: box arena.intern_name("y".to_string()),
            });
        }
        if self.at(Token::Ioverb) {
            let v = try!(self.expect(Token::Ioverb));
            return match v.value()[..1].parse::<u8>() {
                Ok(x) => Ok(K::Ioverb { fd: x }),
                Err(_) => Err(Error::UnexpectedToken),
            };
        }
        if self.at(Token::Bool) {
            let n = try!(self.expect(Token::Bool));
            let v: Vec<K> = n.value()[..n.value().len() - 1]
                .chars()
                .map(|x| K::Int { value: (x as i64) - 0x30 })
                .collect();
            let list = ktree::list(true, &mut arena.ktree, v);
            return self.applyindexright(arena, list);
        }
        if self.at(Token::Hexlit) {
            let h = try!(self.expect(Token::Hexlit));
            return match i64::from_str_radix(&h.value()[2..], 16) {
                Ok(v) => Ok(K::Int { value: v }),
                Err(_) => Err(Error::ParseError(format!("Malformed byte string."))),
            };
        }
        if self.matches(Token::Cond).is_some() {
            return match try!(self.parse_list(arena, Some(Token::CloseB))) {
                K::List { curry: true, values: v } => Ok(K::Condition { list: v }),
                _ => Err(Error::InvalidCondition),
            };
        }
        if self.at(Token::Number) {
            let mut v: Vec<K> = Vec::new();
            while self.at(Token::Number) {
                let n = try!(self.expect(Token::Number));
                let t = try!(n.parse::<i64>());
                v.push(K::Int { value: t });
            }
            return match v.len() {
                1 => self.applyindexright(arena, v.pop().unwrap()),
                _ => {
                    let list = ktree::list(true, &mut arena.ktree, v);
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
                return Ok(ktree::verb(v.chars().nth(0).expect("Char expected."), vec![r]));
            }
            return Ok(ktree::verb(v.chars().nth(0).expect("Char expected."), vec![]));
        }
        if self.at(Token::Symbol) {
            let mut v: Vec<K> = Vec::new();
            while self.at(Token::Symbol) {
                let n = try!(self.expect(Token::Symbol));
                let mut t = try!(n.parse::<String>());
                t.remove(0);
                v.push(arena.intern_symbol(t));
            }
            return match v.len() {
                1 => self.applyindexright(arena, v.pop().unwrap()),
                _ => {
                    let list = ktree::list(true, &mut arena.ktree, v);
                    self.applyindexright(arena, list)
                }
            };
        }
        if self.at(Token::Name) {
            let n = try!(self.expect(Token::Name));
            let t = try!(n.parse::<String>());
            if let Some(x) = self.native(&t) {
                return self.applycallright(arena, x);
            }
            if self.matches(Token::Colon).is_some() {
                let _ = self.matches(Token::Colon).is_some();
                let n = try!(self.parse_noun(arena));
                let r = try!(self.parse_ex(arena, n));
                if r == K::Nil {
                    return Err(Error::ParseError(format!("Noun expected following ':'.")));
                }
                return Ok(K::Nameref {
                    id: arena.intern_name_id(t),
                    value: box r,
                });
            }
            if self.matches(Token::OpenB).is_some() {
                let index = try!(self.parse_list(arena, Some(Token::CloseB)));
                // if (at(ASSIGN)) { return compoundassign(n, index); }
                // if (matches(COLON)) { return indexedassign(n, index); }
                // if (index.length == 0) { index = [NIL]; }
                return Ok(ktree::verb('.', vec![arena.intern_name(t), index]));
            }
            return Ok(arena.intern_name(t));
        }
        if self.matches(Token::OpenB).is_some() {
            let mut keys: Vec<K> = Vec::new();
            let mut values: Vec<K> = Vec::new();
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
            return Ok(K::Dict {
                keys: keys,
                values: values,
            });
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
                r.find_names(&arena.ktree, &mut names);
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
            return self.applycallright(arena,
                                       K::Lambda {
                                           args: args,
                                           body: box r,
                                       });
        }
        if self.matches(Token::OpenP).is_some() {
            let n = try!(self.parse_list(arena, Some(Token::CloseP)));
            return Ok(n);
        }
        Ok(K::Nil)
    }

    fn parse_ex(&mut self, arena: &mut Arena, node: K) -> Result<K, Error> {
        if node == K::Nil {
            return Ok(K::Nil);
        }
        if self.at(Token::Adverb) {
            return self.parse_adverb(arena, K::Nil, node);
        }
        if self.at_noun() {
            let n = try!(self.parse_noun(arena));
            let p = try!(self.parse_ex(arena, n));
            return match node {
                K::Verb { kind: v, args: a } => {
                    if a.is_empty() {
                        Ok(ktree::verb(v as char, vec![p]))
                    } else {
                        Ok(ktree::verb('@', a))
                    }
                }
                x => Ok(ktree::verb('@', vec![x, p])),
            };
        }
        if self.at(Token::Verb) {
            let n = try!(self.expect(Token::Verb));
            let v = n.value();
            if self.at(Token::Adverb) {
                return self.parse_adverb(arena,
                                         node,
                                         ktree::verb(v.chars().nth(0).expect("Char expected."),
                                                     vec![]));
            }
            let x = try!(self.parse_noun(arena));
            let r = try!(self.parse_ex(arena, x));
            return Ok(ktree::verb(v.chars().nth(0).expect("Char expected."), vec![node, r]));
        }
        Ok(node)
    }

    fn parse_list(&mut self, arena: &mut Arena, terminal: Option<Token>) -> Result<K, Error> {
        let mut vec: Vec<K> = Vec::new();
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
            0 => Ok(K::Nil),
            1 => Ok(vec.pop().unwrap()),
            _ => Ok(ktree::list(true, &mut arena.ktree, vec)),
        }
    }
}

pub fn new() -> Parser {
    let natives = Vec::new();
    Parser {
        text: String::new(),
        natives: natives,
    }
}