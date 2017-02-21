#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(concat_idents)]
#![feature(discriminant_value)]
#![feature(slice_patterns)]
#![feature(unboxed_closures)]
#![feature(alloc)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate stacker;
#[macro_use]
extern crate core;
extern crate num;
extern crate alloc;

pub mod parse;
pub mod exec;