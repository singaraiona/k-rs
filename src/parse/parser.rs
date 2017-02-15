use std::str;
use parse::error::Error;
use parse::ktree::{self, K};
use parse::token::{Token, Raw};
use regex::Regex;

pub struct Parser {
    text: String,
    // funcdepth: u16,
    natives: Vec<(String, K)>,
}

impl Parser {
    fn begin(&mut self, s: &str) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(\x22(?:[^\x22\x5C\n]|\.)*\x22)|[a-zA-Z]*(/.*)|([a-z\d\]\)]-\.?\d)|.*")
                                   .unwrap();
        }
        // preserve a string, remove a comment, disambiguate a minus sign.
        self.text = RE.captures_iter(s.trim())
            .map(|cap| {
                if cap.get(1).is_some() {
                    cap[1].to_string()
                } else if cap.get(2).is_some() {
                    str::replace(&cap[0], &cap[2], "")
                } else if cap.get(3).is_some() {
                    str::replace(&cap[3], "-", "- ")
                } else {
                    cap[0].to_string()
                }
            })
            .collect();
        self.text = self.text.replace("\n", ";");
    }

    pub fn parse(&mut self, b: &[u8]) -> Result<K, Error> {
        self.begin(str::from_utf8(b).expect("Invalid input."));
        self.parse_list(None)
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

    fn applycallright(&mut self, node: K) -> Result<K, Error> {
        let mut ret = node;
        while self.matches(Token::OpenB).is_some() {
            let args = try!(self.parse_list(Some(Token::CloseB)));
            ret = ktree::verb(".", vec![ret, args]);
        }
        Ok(ret)
    }

    fn applyindexright(&mut self, node: K) -> Result<K, Error> {
        // if (node.sticky && at(VERB)) {
        //     if self.at(Token::Verb) {
        // 	let x = try!(self.parseNoun());
        //     x.l = node;
        //     let r = try!(self.parse_ex(parseNoun());
        //     return x;
        // }
        let mut r = node;
        while self.matches(Token::OpenB).is_some() {
            let e = try!(self.parse_list(Some(Token::CloseB)));
            r = ktree::verb(".", vec![r, e]);
        }
        return Ok(r);
    }

    fn parse_adverb(&mut self, left: K, verb: K) -> Result<K, Error> {
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
        let n = try!(self.parse_noun());
        let right = try!(self.parse_ex(n));
        return Ok(ktree::adverb(a.value(), box left, box verb, box right));
    }

    #[inline]
    fn native(&self, s: &String) -> Option<K> {
        self.natives.iter().find(|&x| *x.0 == *s).map(|ref x| x.1.clone())
    }

    #[inline]
    fn parse_noun(&mut self) -> Result<K, Error> {
        if self.matches(Token::Colon).is_some() {
            return Ok(K::Lambda {
                args: vec!["x".to_string(), "y".to_string()],
                body: box K::Name { value: "y".to_string() },
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
            return self.applyindexright(K::List { values: v });
        }
        if self.at(Token::Hexlit) {
            let h = try!(self.expect(Token::Hexlit));
            return match i64::from_str_radix(&h.value()[2..], 16) {
                Ok(v) => Ok(K::Int { value: v }),
                Err(_) => Err(Error::ParseError(format!("Malformed byte string."))),
            };
        }
        if self.matches(Token::Cond).is_some() {
            return match try!(self.parse_list(Some(Token::CloseB))) {
                K::List { values: v } => Ok(K::Condition { list: v }),
                _ => Err(Error::InvalidCondition),
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
                let r = try!(self.parse_list(Some(Token::CloseB)));
                return Ok(ktree::verb(v, vec![r]));
            }
            return Ok(ktree::verb(v, vec![]));
        }
        if self.at(Token::Number) {
            let mut v: Vec<K> = Vec::new();
            while self.at(Token::Number) {
                let n = try!(self.expect(Token::Number));
                let t = try!(n.parse::<i64>());
                v.push(K::Int { value: t });
            }
            return match v.len() {
                1 => self.applyindexright(v.pop().unwrap()),
                _ => self.applyindexright(K::List { values: v }),
            };
        }
        if self.at(Token::Symbol) {
            let mut v: Vec<K> = Vec::new();
            while self.at(Token::Symbol) {
                let n = try!(self.expect(Token::Symbol));
                let t = try!(n.parse::<String>());
                v.push(K::Symbol { value: t });
            }
            return match v.len() {
                1 => self.applyindexright(v.pop().unwrap()),
                _ => self.applyindexright(K::List { values: v }),
            };
        }
        if self.at(Token::Name) {
            let n = try!(self.expect(Token::Name));
            let t = try!(n.parse::<String>());
            if let Some(x) = self.native(&t) {
                return self.applycallright(x);
            }
            if self.matches(Token::Colon).is_some() {
                let _ = self.matches(Token::Colon).is_some();
                let n = try!(self.parse_noun());
                let r = try!(self.parse_ex(n));
                if r == K::Nil {
                    return Err(Error::ParseError(format!("Noun expected following ':'.")));
                }
                return Ok(K::Nameref {
                    name: t,
                    value: box r,
                });
            }
            if self.matches(Token::OpenB).is_some() {
                let index = try!(self.parse_list(Some(Token::CloseB)));
                // if (at(ASSIGN)) { return compoundassign(n, index); }
                // if (matches(COLON)) { return indexedassign(n, index); }
                // if (index.length == 0) { index = [NIL]; }
                return Ok(ktree::verb(".", vec![K::Name { value: t }, index]));
            }
            return Ok(K::Name { value: t });
        }
        if self.matches(Token::OpenB).is_some() {
            let mut keys: Vec<K> = Vec::new();
            let mut values: Vec<K> = Vec::new();
            if self.matches(Token::CloseB).is_none() {
                loop {
                    let key = try!(self.expect(Token::Name));
                    let _ = self.expect(Token::Colon);
                    let n = try!(self.parse_noun());
                    let value = try!(self.parse_ex(n));
                    let kname = K::Name { value: try!(key.parse::<String>()) };
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
            let mut args: Vec<String> = Vec::new();
            if self.matches(Token::OpenB).is_some() {
                loop {
                    if self.at(Token::CloseB) {
                        break;
                    }
                    let n = try!(self.expect(Token::Name));
                    let t = try!(n.parse::<String>());
                    args.push(t);
                    if self.matches(Token::Semi).is_none() {
                        break;
                    }
                }
                let _ = try!(self.expect(Token::CloseB));
            }
            let r = try!(self.parse_list(Some(Token::CloseC)));
            if args.is_empty() {
                let mut names: Vec<String> = Vec::new();
                r.find_names(&mut names);
                if names.contains(&"z".to_string()) {
                    args.push(String::from("x"));
                    args.push(String::from("y"));
                    args.push(String::from("z"));
                } else if names.contains(&"y".to_string()) {
                    args.push(String::from("x"));
                    args.push(String::from("y"));
                } else if names.contains(&"x".to_string()) {
                    args.push(String::from("x"));
                }
            }
            return self.applycallright(K::Lambda {
                args: args,
                body: box r,
            });
        }
        if self.matches(Token::OpenP).is_some() {
            let n = try!(self.parse_list(Some(Token::CloseP)));
            return Ok(n);
        }
        Ok(K::Nil)
    }

    fn parse_ex(&mut self, node: K) -> Result<K, Error> {
        if node == K::Nil {
            return Ok(K::Nil);
        }
        if self.at(Token::Adverb) {
            return self.parse_adverb(K::Nil, node);
        }
        if self.at_noun() {
            let n = try!(self.parse_noun());
            let p = try!(self.parse_ex(n));
            return match node {
                K::Verb { kind: v, args: a } => {
                    if a.is_empty() {
                        Ok(ktree::verb(&v[..], vec![p]))
                    } else {
                        Ok(ktree::verb("@", a))
                    }
                }
                x => Ok(ktree::verb("@", vec![x, p])),
            };
        }
        if self.at(Token::Verb) {
            let n = try!(self.expect(Token::Verb));
            let v = n.value();
            if self.at(Token::Adverb) {
                return self.parse_adverb(node, ktree::verb(v, vec![]));
            }
            let x = try!(self.parse_noun());
            let r = try!(self.parse_ex(x));
            return Ok(ktree::verb(v, vec![node, r]));
        }
        Ok(node)
    }

    fn parse_list(&mut self, terminal: Option<Token>) -> Result<K, Error> {
        let mut vec: Vec<K> = Vec::new();
        loop {
            if terminal.is_some() && self.at(terminal.unwrap()) {
                break;
            }
            let n = try!(self.parse_noun());
            match self.parse_ex(n) {
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
            _ => Ok(K::List { values: vec }),
        }
    }
}

pub fn new() -> Parser {
    let mut natives = Vec::new();
    natives.push(("first".to_string(), ktree::verb("*", vec![])));
    Parser {
        text: String::new(),
        // funcdepth: 0,
        natives: natives,
    }
}