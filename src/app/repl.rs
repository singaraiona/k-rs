#![feature(slice_patterns)]
extern crate k;

use k::parse::parser;
use std::io::{self, Read, Write};
use std::str;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn ps1() {
    print!("1>");
    io::stdout().flush().unwrap();
}

fn main() {
    let mut p = parser::new();
    let mut input = vec![0u8; 256];
    println!("K\\ {}", VERSION);
    ps1();
    loop {
        let size = io::stdin().read(&mut input).expect("STDIN error.");
        let k = p.parse(&input[..size - 1]);
        match k {
            Ok(n) => println!("{:#?}", n), 
            Err(e) => println!("Error: {:?}", e),
        }
        ps1();
    }
}