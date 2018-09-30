//! This module defines the test harness which will drive each `Worker`
//! implementation and time how long it takes to run.

use rand::SeedableRng;
use rand::distributions::Distribution;
use rand::distributions::uniform::Uniform;
use rand::rngs::StdRng;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
use worker::{Request, Response, Worker, WorkerReceiver, WorkerSender};

/// The representation of a single request which should be sent to a worker.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RunData {
    /// The amount of time to sleep before sending the request, simulating
    /// a potentially slow client.
    pub sleep: Duration,
    /// The `x` value of a `Request`.
    pub x: usize,
    /// The `y` value of a `Request`.
    pub y: usize,
}

/// Runs the provided worker with generated data, and simulates reader/writer
/// clients of its data.
///
/// # Parameters
/// * `seed` - The seed to use when instantiating the random number generator.
/// Useful for ensuring that different tests run with the same exact data.
/// * `worker` - The `Worker` implementation which should be run.
/// * `my_tx` - A `WorkSender` implementation which the test harness will use
/// to submit requests to the worker. This type must be `Clone`, `Send`, and
/// `'static` so that it can be sent to an arbitrary number of client threads.
/// * `worker_rx` - The receiver handle the worker will use to receive request
/// data.
/// * `worker_tx` - The sender handle the worker will use to send back
/// completed requests.
/// * `my_rx` - The `WorkReceiver` handle which the test harness will use to
/// consume the worker responses.
pub fn run_worker<W, S, R>(
    seed: [u8; 32],
    worker: W,
    my_tx: S,
    worker_rx: W::RequestReceiver,
    worker_tx: W::ResponseSender,
    my_rx: R,
)
    where W: Worker,
          S: 'static + WorkerSender + Clone + Send,
          R: 'static + WorkerReceiver + Send,
{
    const DATA_SIZE: usize = 1000;
    const NUM_CLIENTS: usize = 4;
    const RECEIVER_SLEEP_MS: u64 = 10;

    let dist_sleep_ms = Uniform::new(1, 25);
    let dist_x = Uniform::new(0, 12);
    let dist_y = Uniform::new(1, 1000);

    let rng = &mut StdRng::from_seed(seed);

    let data = (0..DATA_SIZE).into_iter()
        .map(|_| RunData {
            sleep: Duration::from_millis(dist_sleep_ms.sample(rng)),
            x: dist_x.sample(rng),
            y: dist_y.sample(rng),
        })
        .collect();

    run_worker_with_values(
        worker,
        my_tx,
        worker_rx,
        Duration::from_millis(RECEIVER_SLEEP_MS),
        worker_tx,
        my_rx,
        NUM_CLIENTS,
        data
    );
}

/// Runs the provided worker and simulates reader/writer clients of its data.
///
/// # Parameters
/// * `worker` - The `Worker` implementation which should be run.
/// * `my_tx` - A `WorkSender` implementation which the test harness will use
/// to submit requests to the worker. This type must be `Clone`, `Send`, and
/// `'static` so that it can be sent to an arbitrary number of client threads.
/// * `worker_rx` - The receiver handle the worker will use to receive request
/// data.
/// * `receiver_sleep` - The receiver of the worker responses will simulate
/// batching data and will sleep for this long in between each batch.
/// * `worker_tx` - The sender handle the worker will use to send back
/// completed requests.
/// * `my_rx` - The `WorkReceiver` handle which the test harness will use to
/// consume the worker responses.
/// * `num_clients` - The number of client worker threads to use for submitting
/// requests in parallel to the worker.
/// * `run_data` - The data to be used for this test run.
///
/// # Panics
/// `run_data` must be evenly divisible by `num_clients` or a panic will be raised.
pub fn run_worker_with_values<W, S, R>(
    worker: W,
    my_tx: S,
    worker_rx: W::RequestReceiver,
    receiver_sleep: Duration,
    worker_tx: W::ResponseSender,
    mut my_rx: R,
    num_clients: usize,
    mut run_data: Vec<RunData>,
)
    where W: Worker,
          S: 'static + WorkerSender + Clone + Send,
          R: 'static + WorkerReceiver + Send,
{
    // Assert that we can split off our data to our clients evenly
    let run_data_len = run_data.len();
    assert_eq!(run_data_len % num_clients, 0);
    let client_data_size = run_data_len / num_clients;

    let name = worker.name();
    let barrier = Arc::new(Barrier::new(num_clients + 2));
    let mut join_handles = Vec::with_capacity(num_clients + 1);

    // Spawn a number of "client" threads which sleep in between
    for _ in 0..num_clients {
        let barrier_clone = barrier.clone();
        let client_data = run_data.split_off(client_data_size);
        let mut my_tx = my_tx.clone();

        let jh = thread::spawn(move || {
            barrier_clone.wait();

            for data in client_data {
                thread::sleep(data.sleep);
                my_tx.send(Request {
                    x: data.x,
                    y: data.y
                });
            }
        });

        join_handles.push(jh);
    }

    // Ensure we don't have any extra copies of the sender
    // hanging around, or else our worker will never know
    // there is no more data to process.
    drop(my_tx);

    // Spawn the "receiver" thread which will listen for the responses
    let barrier_receiver = barrier.clone();
    join_handles.push(thread::spawn(move || {
        barrier_receiver.wait();

        'outer: loop {
            // Simulate reading the data in batches
            thread::sleep(receiver_sleep);

            for _ in 0..num_clients {
                match my_rx.recv() {
                    Some(Response { x, y, result }) => {
                        trace!("{}: x: {}, y: {}, result = {}", name, x, y, result);
                    },
                    None => break 'outer,
                }
            }
        }
    }));

    println!("\nrunning {}", name);

    // Synchronize with our "client" threads
    barrier.wait();

    let start = Instant::now();
    worker.do_work(worker_rx, worker_tx);
    let end = Instant::now();

    println!("{} took {:#?} to run to completion", name, end - start);

    // Ensure all of our client threads are shut down, so they don't interfere
    // with future runs
    for jh in join_handles {
        jh.join().expect("failed to join client thread");
    }
}
