//! This module provides a `Worker` implementation which uses a rayon
//! parallel iterator to perform work.

use harness::run_worker;
use std::sync::mpsc::{channel, Sender, Receiver};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use worker::{compute_response, Request, Response, Worker};

/// A `Worker` implementation which uses its own blocking threadpool
/// to process data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RayonWorker(());

impl RayonWorker {
    fn new() -> Self {
        RayonWorker(())
    }
}

impl Default for RayonWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Worker for RayonWorker {
    type RequestReceiver = Receiver<Request>;
    type ResponseSender = Sender<Response>;

    fn name(&self) -> &'static str {
        "rayon worker"
    }

    fn do_work(self, rx: Self::RequestReceiver, resp_tx: Self::ResponseSender) {
        // Rayon only supports parallel iterators which can be indexed
        // independently. In other words, a sequential iterator like a `Receiver`
        // cannot support parallel operations, so we'll need to buffer all
        // the requests into a vector before we can process them in parallel.
        let requests: Vec<_> = rx.into_iter().collect();

        // Similarly, we can't dispatch all the responses into the `Sender`
        // channel because it isn't `Sync`. Instead of using locks on the sender
        // we'll buffer all the responses and dispatch them at once.
        let responses: Vec<_> = requests.into_par_iter()
            .map(compute_response)
            .collect();

        for resp in responses {
            if resp_tx.send(resp).is_err() {
                break
            }
        }
    }
}

/// Set up and run the test harness for a `RayonWorker`.
pub fn run(seed: [u8; 32]) {
    let (harness_tx, worker_rx) = channel();
    let (worker_tx, harness_rx) = channel();

    run_worker(
        seed,
        RayonWorker::new(),
        move || harness_tx.clone(),
        worker_rx,
        worker_tx,
        harness_rx,
    );
}
