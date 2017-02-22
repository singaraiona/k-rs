#![feature(slice_patterns)]
#![feature(test)]

extern crate test;
extern crate k;

use k::parse::ast::{AST, pp};
use k::exec::i10;
use std::io::{self, Read, Write};
use std::str;
use std::ascii::AsciiExt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn ps1() {
    print!("o)");
    io::stdout().flush().unwrap();
}

fn main() {
    let mut i = i10::new();
    let mut input = vec![0u8; 256];
    println!("Welcome to O lang v{} interpreter...", VERSION);
    ps1();
    loop {
        let size = io::stdin().read(&mut input).expect("STDIN error.");
        let k = i.parse(&input[..size - 1]);
        match k {
            Ok(n) => {
                println!("------ Parse ------ \n{:#?}", n);
                match i.run(&n) {
                    Ok(x) => {
                        match x {
                            AST::Quit => break,
                            u => {
                                pp(&u, i.arena());
                                println!("");
                            }
                        }
                    }
                    Err(e) => println!("'{}", format!("{:?}", e).to_ascii_lowercase()),
                }
            }
            Err(e) => println!("'{}", format!("{:?}", e).to_ascii_lowercase()),
        }
        ps1();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn fac_k(b: &mut Bencher) {
        let mut i = i10::new();
        let code = i.parse(b"fac:{$[x=1;1;x*fac[x-1]]}").unwrap();
        let _ = i.run(&code);
        let f = i.parse(b"fac[5]").unwrap();
        b.iter(|| {
            let _ = i.run(&f);
            i.gc();
        });
    }
}