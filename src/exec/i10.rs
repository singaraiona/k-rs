use parse::ktree::K;
use exec::error::Error;

fn add(x: &K, y: &K) -> Result<K, Error> {
    Ok(K::Int { value: 1 + 2 })
}

fn am(v: &str, x: &K) -> Result<K, Error> {
    Ok(K::Int { value: 6 })
}

fn ad(v: &str, x: &K, y: &K) -> Result<K, Error> {
    Ok(K::List { values: vec![x.clone(), y.clone()] })
}

fn at(v: &str, x: &K, y: &K, z: &K) -> Result<K, Error> {
    Ok(K::List { values: vec![x.clone(), y.clone(), z.clone()] })
}

macro_rules! call {
    ($f:expr,$i:ident) => (match &$i[..] {
        &[ref x] => am($f, x),
        &[ref x, ref y] => ad($f, x,y),
        &[ref x, ref y, ref z] => at($f, x,y,z),
        _ => Err(Error::Rank),
    })
}

pub struct Interpreter {
    
}