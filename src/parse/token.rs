use regex::Regex;
use std::str::FromStr;
use parse::error::Error;

lazy_static! {
    // Tokens
    static ref TOKENS: Vec<Regex> = vec![r"^[01]+b",                               // 0 - BOOL
                                         r"^0x[a-zA-Z\d]+",                        // 1 - HEXLIT
                                         r"^-?(0w|0N|\d+\.\d*|\d*\.?\d)",          // 2 - NUMBER
                                         r"^[a-z][a-z\d]*",                        // 3 - NAME
                                         r"^`([a-zA-Z0-9.]*)?",                    // 4 - SYMBOL 
                                         r"^\x22(\\.|[^\x5C\x22])*\x22",           // 5 - CHAR
                                         r"^[+\x2D*%!&|<>=~,^#_$?@.]",             // 6 - VERB 
                                         r"^[+\x2D*%!&|<>=~,^#_$?@.]:",            // 7 - ASSIGN
                                         r"^\d:",                                  // 8 - IOVERB
                                         r"^['\x5c\x2f]+:?",                       // 9 - ADVERB
                                         r"^;",                                    // 10- SEMI
                                         r"^:",                                    // 11- COLON
                                         r"^::",                                   // 12- VIEW 
                                         r"^\$\[",                                 // 13- COND
                                         r"^\[[a-z]+:",                            // 14- DICT  
                                         r"^\[",                                   // 15- OPEN_B
                                         r"^\(",                                   // 16- OPEN_P
                                         r"^\{",                                   // 17- OPEN_C 
                                         r"^\]",                                   // 18- CLOSE_B 
                                         r"^\)",                                   // 19- CLOSE_P 
                                         r"^}",                                    // 20- CLOSE_C
                                         r"^\\\\",]                                // 21- QUIT 
                                         .iter().map(|x| Regex::new(x).unwrap()).collect();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Token {
    Bool,
    Hexlit,
    Number,
    Name,
    Symbol,
    Char,
    Verb,
    Assign,
    Ioverb,
    Adverb,
    Semi,
    Colon,
    View,
    Cond,
    Dict,
    OpenB,
    OpenP,
    OpenC,
    CloseB,
    CloseP,
    CloseC,
    Quit,
    Nil,
}

impl Token {
    #[inline]
    fn re(&self) -> &Regex {
        TOKENS.get(*self as usize).expect("Invalid token.")
    }

    pub fn find(&self, s: &str) -> Option<Raw> {
        match self.re().find(s) {
            Some(m) => Some(Raw(s[m.start()..m.end()].to_string())),
            None => None,
        }
    }

    pub fn is_match(&self, s: &str) -> bool {
        self.re().is_match(s)
    }
}

#[derive(Debug)]
pub struct Raw(String);

impl Raw {
    pub fn parse<T: FromStr>(&self) -> Result<T, Error> {
        match self.0.parse::<T>() {
            Ok(t) => Ok(t),
            Err(_) => Err(Error::ParseError(format!("Can not parse type from {}", self.0))),
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}