#![feature(slice_patterns)]
extern crate k;

use k::parse::parser;
use k::exec::i10::Interpreter;
use std::io::{self, Read, Write};
use std::str;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn ps1() {
    print!("1>");
    io::stdout().flush().unwrap();
}

fn main() {
    let mut p = parser::new();
    let mut i = Interpreter::new();
    let mut input = vec![0u8; 256];

    let v1 = [1, 2, 3, 4];
    let v2 = 7;

    let r = v1.iter().map(|x| x + v2);
    println!("R: {:?}", r);

    println!("K\\ {}", VERSION);
    ps1();
    loop {
        let size = io::stdin().read(&mut input).expect("STDIN error.");
        let k = p.parse(&input[..size - 1]);
        match k {
            Ok(n) => {
                println!("------ Parse ------ \n{:#?}", n);
                let r = i.run(n);
                println!("------ Run ------ \n{:?}", r);
            } 
            Err(e) => println!("Error: {:?}", e),
        }
        ps1();
    }
}