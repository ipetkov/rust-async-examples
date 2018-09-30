//! This repo shows a few different ways of doing parallel/async computations
//! in Rust, with the use of several different community crates.

#![deny(missing_docs)]

extern crate env_logger;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate rand;

use rand::RngCore;

pub mod harness;
pub mod sequential;
pub mod threadpool;
pub mod worker;

fn main() {
    env_logger::init();

    let seed = {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        seed
    };

    sequential::run(seed.clone());
    threadpool::run(seed.clone());
}
