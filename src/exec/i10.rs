use parse::ktree::K;
use parse::error::Error as ParseError;
use exec::error::Error as ExecError;
use exec::env::Environment;

fn add(left: &K, right: &K, env: &mut Environment) -> Result<K, ExecError> {
    match (left, right) {
        (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Int { value: a + b }),
        (&K::List { values: ref a }, &K::Int { value: b }) => {
            let mut r: Vec<K> = Vec::new();
            for x in a.iter() {
                r.push(try!(add(x, &K::Int { value: b }, env)));
            }
            return Ok(K::List { values: r });
        }
        (&K::Int { value: a }, &K::List { values: ref b }) => {
            let mut r: Vec<K> = Vec::new();
            for x in b.iter() {
                r.push(try!(add(x, &K::Int { value: a }, env)));
            }
            return Ok(K::List { values: r });
        }
        (&K::List { values: ref a }, &K::List { values: ref b }) => {
            if a.len() != b.len() {
                return Err(ExecError::Length);
            }
            let mut r: Vec<K> = Vec::new();
            for (x, y) in a.iter().zip(b.iter()) {
                r.push(try!(add(x, y, env)));
            }
            return Ok(K::List { values: r });
        }
        _ => (),
    };
    Err(ExecError::Type)
}

fn eq(left: &K, right: &K, env: &mut Environment) -> Result<K, ExecError> {
    match (left, right) {
        (&K::Int { value: a }, &K::Int { value: b }) => return Ok(K::Bool { value: a == b }),
        _ => (),
    };
    Err(ExecError::Type)
}

fn cond(c: &[K], env: &mut Environment) -> Result<K, ExecError> {
    match c {
        &[ref e, ref x, ref y] => {
            match try!(run(&e, env)) {
                K::Bool { value: b } => {
                    if b {
                        return run(&x, env);
                    }
                    return run(&y, env);
                }
                _ => Err(ExecError::Condition),
            }
        }
        _ => Err(ExecError::Condition),
    }
}

fn call(lambda: &K, args: &[K], env: &mut Environment) -> Result<K, ExecError> {
    match lambda {
        &K::Lambda { args: ref a, body: ref b } => {
            for (n, v) in a.iter().zip(args.iter()) {
                define(n, v, env);
            }
            return run(b, env);
        }
        _ => (),
    }
    Err(ExecError::Call)
}

fn apply(lambda: &K, args: &[K], env: &mut Environment) -> Result<K, ExecError> {
    call(lambda, args, env)
}

fn define(name: &str, value: &K, env: &mut Environment) -> Result<K, ExecError> {
    env.define(name, value);
    Ok(value.clone())
}

fn get(name: &str, env: &mut Environment) -> Result<K, ExecError> {
    match env.get(name) {
        Some(n) => Ok(n),
        None => Ok(K::Nil),
    }
}

pub fn run(k: &K, env: &mut Environment) -> Result<K, ExecError> {
    match *k {
        K::Verb { kind: ref k, args: ref a } => {
            match &k[..] {
                "+" => {
                    let x = try!(run(&a[0], env));
                    let y = try!(run(&a[1], env));
                    return add(&x, &y, env);
                }
                "=" => {
                    let x = try!(run(&a[0], env));
                    let y = try!(run(&a[1], env));
                    return eq(&x, &y, env);
                }
                "." => {
                    let x = try!(run(&a[0], env));
                    return call(&x, &a[1..], env);
                }
                "@" => {
                    let x = try!(run(&a[0], env));
                    return apply(&x, &a[1..], env);
                }
                _ => (),
            };
        }
        K::Condition { list: ref c } => return cond(c, env),
        K::Nameref { name: ref n, value: ref v } => return define(&n[..], v, env),
        K::Name { value: ref n } => return get(n, env),        
        _ => return Ok(k.clone()),
    };
    Ok(K::Nil)
}