#![feature(slice_patterns)]
#![feature(test)]

extern crate test;
extern crate k;

use k::parse::ktree::pp;
use k::exec::i10;
use std::io::{self, Read, Write};
use std::str;
use std::ascii::AsciiExt;
use k::exec::env::Environment;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn ps1() {
    print!("1>");
    io::stdout().flush().unwrap();
}

fn main() {
    let mut i = i10::new();
    let mut env = Environment::new();
    let mut input = vec![0u8; 256];
    println!("K\\ {}", VERSION);
    ps1();
    loop {
        let size = io::stdin().read(&mut input).expect("STDIN error.");
        let k = i.parse(&input[..size - 1]);
        match k {
            Ok(n) => {
                // println!("------ Parse ------ \n{:#?}", n);
                match i.run(&n, env.clone()) {
                    Ok(x) => {
                        pp(&x, i.arena());
                        println!("");
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
        let mut env = Environment::new();
        let code = i.parse(b"fac:{$[x=1;1;x*fac[x-1]]}").unwrap();
        i.run(&code, env.clone());
        let f = i.parse(b"fac[5]").unwrap();
        b.iter(|| {
            let _ = i.run(&f, env.clone());
        });
    }
}