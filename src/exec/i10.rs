use parse::ktree::K;
use parse::error::Error as ParseError;
use exec::error::Error as ExecError;

fn add(left: &K, right: &K) -> Result<K, ExecError> {
    match (left, right) {
        (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a + b }),
        (&K::List { values: ref a }, &K::Int { value: b }) => {
            let mut r: Vec<K> = Vec::new();
            for x in a.iter() {
                r.push(try!(add(x, &K::Int { value: b })));
            }
            return Ok(K::List { values: r });
        }
        (&K::Int { value: a }, &K::List { values: ref b }) => {
            let mut r: Vec<K> = Vec::new();
            for x in b.iter() {
                r.push(try!(add(x, &K::Int { value: a })));
            }
            return Ok(K::List { values: r });
        }
        (&K::List { values: ref a }, &K::List { values: ref b }) => {
            if a.len() != b.len() {
                return Err(ExecError::Length);
            }
            let mut r: Vec<K> = Vec::new();
            for (x, y) in a.iter().zip(b.iter()) {
                r.push(try!(add(x, y)));
            }
            return Ok(K::List { values: r });
        }
        _ => (),
    };
    Err(ExecError::Type)
}

fn eq(left: &K, right: &K) -> Result<K, ExecError> {
    match (left, right) {
        (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Bool { value: a == b }),
        _ => (),
    };
    Err(ExecError::Type)
}

fn cond(c: &[K]) -> Result<K, ExecError> {
    match c {
        &[ref e, ref x, ref y] => {
            match try!(run(&e)) {
                K::Bool { value: b } => {
                    if b {
                        return run(&x);
                    }
                    return run(&y);
                }
                _ => Err(ExecError::Condition),
            }
        }
        _ => Err(ExecError::Condition),
    }
}

pub fn run(k: &K) -> Result<K, ExecError> {
    match *k {
        K::Verb { kind: ref k, args: ref a } => {
            match &k[..] {
                "+" => {
                    let x = try!(run(&a[0]));
                    let y = try!(run(&a[1]));
                    return add(&x, &y);
                }
                "=" => {
                    let x = try!(run(&a[0]));
                    let y = try!(run(&a[1]));
                    return eq(&x, &y);
                }
                _ => (),
            };
        }
        K::Condition { list: ref c } => return cond(c),
        _ => return Ok(k.clone()),
    };
    Ok(K::Nil)
}