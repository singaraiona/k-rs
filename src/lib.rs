#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(concat_idents)]
#![feature(discriminant_value)]
#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

pub mod parse;
pub mod exec;