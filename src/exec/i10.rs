use parse::ktree::K;
use parse::error::Error as ParseError;
use exec::error::Error as ExecError;

// fn add(x: &K, y: &K) -> Result<K, Error> {
//     Ok(K::Int { value: 1 + 2 })
// }

// fn am(v: &str, x: &K) -> Result<K, Error> {
//     Ok(K::Int { value: 6 })
// }

// fn ad(v: &str, x: &K, y: &K) -> Result<K, Error> {
//     Ok(K::List { values: vec![x.clone(), y.clone()] })
// }

// fn at(v: &str, x: &K, y: &K, z: &K) -> Result<K, Error> {
//     Ok(K::List { values: vec![x.clone(), y.clone(), z.clone()] })
// }

// macro_rules! call {
//     ($f:expr,$i:ident) => (match &$i[..] {
//         &[ref x] => am($f, x),
//         &[ref x, ref y] => ad($f, x,y),
//         &[ref x, ref y, ref z] => at($f, x,y,z),
//         _ => Err(Error::Rank),
//     })
// }

pub struct Interpreter {
    }

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn run(&mut self, mut k: K) -> Result<K, ExecError> {

        Ok(K::Nil)
    }
}