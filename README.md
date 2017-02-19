K-RS
=======

An open-source interpreter for the K6 programming language.

Currently it requires nightly version of Rust. Test it:
-----------------------------------------------------------

``` 
git clone git@github.com:singaraiona/k-rs.git
cd k-rs
cargo run
```

Basic functionality:
--------------------

```
K\ 0.1.1
1>f:{{x+3}x}
{[x]{[x]x+3}@x}
1>f 12
15
```

Links: 
------

* [K6 specification.](http://www.kparc.com)
* [An open-source interpreter for the K5 programming language.](http://git@github.com:JohnEarnest/ok.git)