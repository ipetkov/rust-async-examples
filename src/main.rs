//! This repo shows a few different ways of doing parallel/async computations
//! in Rust, with the use of several different community crates.

#![deny(missing_docs)]

extern crate env_logger;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate rand;

pub mod harness;
pub mod sequential;
pub mod threadpool;
pub mod worker;

fn main() {
    env_logger::init();

    sequential::run();
    threadpool::run();
}
