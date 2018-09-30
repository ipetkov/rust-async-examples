//! This repo shows a few different ways of doing parallel/async computations
//! in Rust, with the use of several different community crates.
//!
//! The test harness is pretty contrived, so don't expect to get any meaningful
//! comparisons of the performance of each method. However, the code samples
//! should be somewhat indicative of the different paradigms.
//!
//! To see the examples in action, simply run `cargo run --release`!

#![deny(missing_docs)]

extern crate env_logger;
extern crate futures;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate rand;
extern crate rayon;
extern crate tokio;

use rand::RngCore;

pub mod harness;
pub mod par_iter;
pub mod sequential;
pub mod threadpool;
pub mod tokio_worker;
pub mod worker;

fn main() {
    env_logger::init();

    let seed = {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        seed
    };

    sequential::run(seed);
    threadpool::run(seed);
    par_iter::run(seed);
    tokio_worker::run_current_thread(seed);
    tokio_worker::run(seed);
}
