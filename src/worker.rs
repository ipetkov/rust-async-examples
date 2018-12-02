//! This module defines the interfaces that will be used by various
//! parallel/async worker implementations.

use futures::sink;
use futures::stream;
use futures::sync::mpsc::{Receiver as FuturesReceiver, Sender as FuturesSender};
use std::sync::mpsc::{Receiver, Sender};

/// Represents a request that is send to the worker for processing.
///
/// The worker should compute `x! / y`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Request {
    /// The value whose factorial should be computed.
    pub x: usize,
    /// The divisor of the operation.
    pub y: usize,
}

/// Represents a unit of work completed by the worker.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Response {
    /// The value whose factorial should be computed.
    pub x: usize,
    /// The divisor of the operation.
    pub y: usize,
    /// The final result of the `x! / y` operation.
    pub result: usize,
}

/// Abstraction over the Sender end of different channel implementations.
///
/// The test harness will use this abstraction to send data over to the worker.
pub trait WorkerSender {
    /// Send the data over to the worker.
    fn send(&mut self, request: Request);
}

impl<'a, T: WorkerSender> WorkerSender for &'a mut T {
    fn send(&mut self, request: Request) {
        (**self).send(request);
    }
}

impl WorkerSender for Sender<Request> {
    fn send(&mut self, request: Request) {
        let _ = (*self).send(request);
    }
}

impl WorkerSender for sink::Wait<FuturesSender<Request>> {
    fn send(&mut self, request: Request) {
        let _ = (*self).send(request);
    }
}

/// Abstraction over the Receiver end of different channel implementations.
///
/// The test harness will use this abstraction to receive data over from the worker.
pub trait WorkerReceiver {
    /// Receive the data from to the worker.
    ///
    /// When no more data is available, `None` should be returned.
    fn recv(&mut self) -> Option<Response>;
}

impl<'a, T: WorkerReceiver> WorkerReceiver for &'a mut T {
    fn recv(&mut self) -> Option<Response> {
        (**self).recv()
    }
}

impl WorkerReceiver for Receiver<Response> {
    fn recv(&mut self) -> Option<Response> {
        (*self).recv().ok()
    }
}

impl WorkerReceiver for stream::Wait<FuturesReceiver<Response>> {
    fn recv(&mut self) -> Option<Response> {
        self.next().and_then(Result::ok)
    }
}

/// An interface which represents some type of worker which will receive some input,
/// do some processing, and then transmit the result back.
///
/// This will allow us to define a single test harness for invoking our different
/// parallelization strategies.
pub trait Worker {
    /// The channel receiver type which the worker will use to receive input
    /// from the test harness.
    type RequestReceiver;
    /// The channel sender type which the worker will use to send output back
    /// to the test harness.
    type ResponseSender;

    /// Get the name of this worker so we can print out how long it takes to run.
    fn name(&self) -> &'static str;

    /// Consume the worker and let them do the actual work.
    ///
    /// While requests continue to come through `rx`, the worker should process
    /// the requests and send a response through `tx`.
    fn do_work(self, rx: Self::RequestReceiver, tx: Self::ResponseSender);
}

/// Computes `x! / y` using a provided `Request`.
///
/// # Panics
/// Panics if `request.y == 0`.
pub fn compute_response(request: Request) -> Response {
    assert_ne!(request.y, 0, "divisor cannot be 0!");

    let mut fac = 1;

    for i in 1..request.x {
        fac *= i;
    }

    Response {
        x: request.x,
        y: request.y,
        result: fac / request.y,
    }
}
