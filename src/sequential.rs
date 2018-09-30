//! This module provides a `Worker` implementation which sequentially processes data.

use harness::run_worker;
use std::sync::mpsc::{channel, Sender, Receiver};
use worker::{compute_response, Request, Response, Worker};

/// A `Worker` implementation which sequentially processes data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequentialWorker(());

impl SequentialWorker {
    fn new() -> Self {
        SequentialWorker(())
    }
}

impl Default for SequentialWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Worker for SequentialWorker {
    type RequestReceiver = Receiver<Request>;
    type ResponseSender = Sender<Response>;

    fn name(&self) -> &'static str {
        "sequential worker"
    }

    fn do_work(self, rx: Self::RequestReceiver, tx: Self::ResponseSender) {
        while let Ok(req) = rx.recv() {
            if tx.send(compute_response(req)).is_err() {
                break
            }
        }
    }
}

/// Set up and run the test harness for a `SequentialWorker`.
pub fn run(seed: [u8; 32]) {
    let (harness_tx, worker_rx) = channel();
    let (worker_tx, harness_rx) = channel();

    run_worker(
        seed,
        SequentialWorker::new(),
        harness_tx,
        worker_rx,
        worker_tx,
        harness_rx,
    );
}
