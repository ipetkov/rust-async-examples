//! This repo shows a few different ways of doing parallel/async computations
//! in Rust, with the use of several different community crates.

#![deny(missing_docs)]

extern crate rand;

pub mod harness;
pub mod sequential;
pub mod worker;

fn main() {
    sequential::run();
}
