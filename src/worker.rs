//! This module defines the interfaces that will be used by various
//! parallel/async worker implementations.

/// Represents a request that is send to the worker for processing.
///
/// The worker should compute `x! / y`
pub struct Request {
    /// The value whose factorial should be computed.
    pub x: usize,
    /// The divisor of the operation.
    pub y: usize,
}

/// Represents a unit of work completed by the worker.
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

/// Abstraction over the Receiver end of different channel implementations.
///
/// The test harness will use this abstraction to receive data over from the worker.
pub trait WorkerReceiver {
    /// Receive the data from to the worker.
    ///
    /// When no more data is available, `None` should be returned.
    fn recv(&mut self) -> Option<Response>;
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
