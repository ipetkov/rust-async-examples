//! This module provides a `Worker` implementation which uses futures
//! and tokio to perform work.

use futures::future::ok;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Future, Sink, Stream};
use harness::run_worker;
use tokio;
use worker::{compute_response, Request, Response, Worker};

/// A `Worker` implementation which uses its own a tokio runtime
/// to process futures with a threadpool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokioWorker(());

impl TokioWorker {
    fn new() -> Self {
        TokioWorker(())
    }
}

impl Default for TokioWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Worker for TokioWorker {
    type RequestReceiver = Receiver<Request>;
    type ResponseSender = Sender<Response>;

    fn name(&self) -> &'static str {
        "tokio worker"
    }

    fn do_work(self, rx: Self::RequestReceiver, resp_tx: Self::ResponseSender) {
        tokio::run(process(rx, resp_tx));
    }
}

/// A `Worker` implementation which uses its own a tokio `CurrentThread`
/// runtime to process futures with a single thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokioCurrentThreadWorker(());

impl TokioCurrentThreadWorker {
    fn new() -> Self {
        TokioCurrentThreadWorker(())
    }
}

impl Default for TokioCurrentThreadWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl Worker for TokioCurrentThreadWorker {
    type RequestReceiver = Receiver<Request>;
    type ResponseSender = Sender<Response>;

    fn name(&self) -> &'static str {
        "tokio current thread worker"
    }

    fn do_work(self, rx: Self::RequestReceiver, resp_tx: Self::ResponseSender) {
        let mut rt = tokio::runtime::current_thread::Runtime::new()
            .expect("failed to create a `current_thread::Runtime`");

        // Run the current future to completion, blocking the current thread.
        rt.block_on(process(rx, resp_tx))
            .expect("failed to run successfully");
    }
}

/// Set up and run the test harness for a `TokioWorker`.
pub fn run(seed: [u8; 32]) {
    run_with_worker(seed, TokioWorker::new());
}

/// Set up and run the test harness for a `TokioCurrentThreadWorker`.
pub fn run_current_thread(seed: [u8; 32]) {
    run_with_worker(seed, TokioCurrentThreadWorker::new());
}

fn run_with_worker<W>(seed: [u8; 32], worker: W)
where
    W: Worker<RequestReceiver = Receiver<Request>, ResponseSender = Sender<Response>>,
{
    // NB: by default futures channels reserve at least one slot per sender
    let (harness_tx, worker_rx) = channel(0);
    let (worker_tx, harness_rx) = channel(4);

    run_worker(
        seed,
        worker,
        move || harness_tx.clone().wait(),
        worker_rx,
        worker_tx,
        harness_rx.wait(),
    );
}

fn process(
    rx: Receiver<Request>,
    resp_tx: Sender<Response>,
) -> impl Future<Item = (), Error = ()> + 'static + Send {
    // Process the input stream by computing the responses
    let input_stream = rx
        .map(|req| ok(compute_response(req)))
        .map_err(|_| ()) // Ignore any receive errors
        // Buffer unordered will allow up to N futures to execute
        // concurrently in the same runtime (i.e. if one future isn't
        // ready to make progress, the executor will switch to running
        // another future which is ready).
        .buffer_unordered(4);

    // Ignore any send errors
    let output_sink = resp_tx.sink_map_err(|_| ());

    // A future which completes once the entire input stream is
    // sent into the output sink.
    output_sink
        .send_all(input_stream)
        // The combinators here eventually return the inner types
        // so they can be used further, but since we don't plan to
        // we'll simplify our return type to just ().
        .map(|_| ())
}
