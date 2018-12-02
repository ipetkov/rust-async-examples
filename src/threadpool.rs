//! This module provides a `Worker` implementation which uses its own
//! blocking threadpool to process data.

use harness::run_worker;
use num_cpus;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use worker::{compute_response, Request, Response, Worker};

/// A `Worker` implementation which uses its own blocking threadpool
/// to process data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThreadPoolWorker(());

impl ThreadPoolWorker {
    fn new() -> Self {
        ThreadPoolWorker(())
    }
}

impl Default for ThreadPoolWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Worker for ThreadPoolWorker {
    type RequestReceiver = Receiver<Request>;
    type ResponseSender = Sender<Response>;

    fn name(&self) -> &'static str {
        "threadpool worker"
    }

    fn do_work(self, rx: Self::RequestReceiver, resp_tx: Self::ResponseSender) {
        // Although there are plenty of great crates which implement
        // efficient threadpools, we'll implement our own simple one for
        // illustrative purposes. Reusing the same worker threads will be much
        // efficient than spinning up a new thread for each incoming request.
        let num_cpus = num_cpus::get();
        let mut join_handles = Vec::with_capacity(num_cpus);

        // We're using the standard library's mpsc channel, which stands for
        // multi-producer, single-consumer channel. This means that we can
        // have multiple instances producing data, which will go to a single
        // receiver.
        //
        // In other words, we can keep cloning our sender half to each of our
        // workers and they will all send data to the single receiver client
        // of the test harness. However, we cannot clone the request receiver
        // which was passed into this worker. To get around this, we'll have
        // the worker proxy the requests in a round robin fashion.
        let thread_senders = {
            let mut thread_senders = Vec::with_capacity(num_cpus);

            for _ in 0..num_cpus {
                let (parent_tx, worker_rx) = channel();
                thread_senders.push(parent_tx);

                let resp_tx = resp_tx.clone();
                let jh = thread::spawn(move || thread_worker(worker_rx, resp_tx));
                join_handles.push(jh);
            }

            thread_senders
        };

        // Ensure we drop the sender half we're still holding on
        // otherwise the receiver won't know that all senders were
        // dropped after our worker threads have finished.
        drop(resp_tx);

        // Although this code looks very similar to our sequential implementation,
        // we're gaining some throughput by having *multiple* threads which can be
        // processing data or blocked on sending the response through.
        let mut iter = (0..thread_senders.len()).cycle();
        while let Ok(req) = rx.recv() {
            let next_sender_idx = iter.next().expect("should never get none on cycle");
            let _ = thread_senders[next_sender_idx].send(req);
        }

        // Drop the channel handles to the workers which will signal them to exit.
        drop(thread_senders);

        // Ensure we clean up any threads we spawned
        for jh in join_handles {
            jh.join().expect("failed to join worker thread");
        }
    }
}

fn thread_worker(rx: Receiver<Request>, tx: Sender<Response>) {
    while let Ok(req) = rx.recv() {
        if tx.send(compute_response(req)).is_err() {
            break;
        }
    }
}

/// Set up and run the test harness for a `ThreadPoolWorker`.
pub fn run(seed: [u8; 32]) {
    let (harness_tx, worker_rx) = channel();
    let (worker_tx, harness_rx) = channel();

    run_worker(
        seed,
        ThreadPoolWorker::new(),
        move || harness_tx.clone(),
        worker_rx,
        worker_tx,
        harness_rx,
    );
}
