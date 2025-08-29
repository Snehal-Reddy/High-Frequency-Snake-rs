use criterion::{Criterion, criterion_group, criterion_main};
use high_frequency_snake::game::types::{Direction, Input};
use high_frequency_snake::ipc::spsc::Spsc;
use std::hint::black_box;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const QUEUE_CAPACITY: usize = 65536;
const NUM_MESSAGES: usize = 1_000_000;
const BUSY_SPIN_ITERS: u64 = 100; // Tunable work simulation

/// A function to simulate CPU-bound work that the compiler cannot optimize away.
#[inline(never)]
fn busy_spin(iters: u64) {
    for i in 0..iters {
        black_box(i);
    }
}

// Not very accurate
fn spsc_throughput_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_throughput");
    group.throughput(criterion::Throughput::Elements(NUM_MESSAGES as u64));

    group.bench_function("multi_threaded_throughput", |b| {
        let queue = Arc::new(Spsc::<Input, QUEUE_CAPACITY>::new());
        let (start_tx, start_rx) = channel::<()>();
        let (done_tx, done_rx) = channel::<()>();
        let start_rx = Arc::new(Mutex::new(start_rx));

        let producer_queue = Arc::clone(&queue);
        let producer_done_tx = done_tx.clone();
        let producer_start_rx = Arc::clone(&start_rx);
        thread::spawn(move || {
            let input = Input {
                snake_id: 1,
                direction: Direction::Up,
            };
            loop {
                if producer_start_rx.lock().unwrap().recv().is_err() {
                    break;
                }
                for _ in 0..NUM_MESSAGES {
                    while !producer_queue.produce(input) {
                        thread::yield_now();
                    }
                }
                producer_done_tx.send(()).unwrap();
            }
        });

        let consumer_done_tx = done_tx;
        let consumer_start_rx = Arc::clone(&start_rx);
        thread::spawn(move || {
            loop {
                if consumer_start_rx.lock().unwrap().recv().is_err() {
                    break;
                }
                for _ in 0..NUM_MESSAGES {
                    while queue.consume().is_none() {
                        thread::yield_now();
                    }
                }
                consumer_done_tx.send(()).unwrap();
            }
        });

        b.iter(|| {
            start_tx.send(()).unwrap();
            start_tx.send(()).unwrap();
            done_rx.recv().unwrap();
            done_rx.recv().unwrap();
        });
    });

    group.finish();
}

// Prolly more accurate
fn spsc_latency_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_latency");

    let core_ids = core_affinity::get_core_ids();
    if core_ids.is_none() || core_ids.as_ref().unwrap().len() < 2 {
        println!("Skipping latency test: at least 2 CPU cores required.");
        return;
    }
    let core_a = core_ids.as_ref().unwrap()[0];
    let core_b = core_ids.as_ref().unwrap()[1];

    group.bench_function("ping_pong_rtt", |b| {
        let ping_queue = Arc::new(Spsc::<Instant, QUEUE_CAPACITY>::new());
        let pong_queue = Arc::new(Spsc::<Instant, QUEUE_CAPACITY>::new());

        let pong_ping_queue = Arc::clone(&ping_queue);
        let pong_pong_queue = Arc::clone(&pong_queue);
        thread::spawn(move || {
            core_affinity::set_for_current(core_b);
            loop {
                if let Some(timestamp) = pong_ping_queue.consume() {
                    while !pong_pong_queue.produce(timestamp) {
                        thread::yield_now();
                    }
                }
            }
        });

        b.iter_custom(|iters| {
            core_affinity::set_for_current(core_a);
            let mut total_duration = std::time::Duration::new(0, 0);
            for _ in 0..iters {
                let start = Instant::now();
                while !ping_queue.produce(start) {
                    thread::yield_now();
                }
                loop {
                    if let Some(received_start) = pong_queue.consume() {
                        total_duration += received_start.elapsed();
                        break;
                    }
                }
            }
            total_duration
        });
    });

    group.finish();
}

// Hmmm not sure about this
fn spsc_contention_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_contention");
    group.throughput(criterion::Throughput::Elements(NUM_MESSAGES as u64));

    // --- Producer is faster than the consumer (queue is often full) ---
    group.bench_function("producer_faster_consumer_slower", |b| {
        let queue = Arc::new(Spsc::<Input, QUEUE_CAPACITY>::new());
        let (start_tx, start_rx) = channel::<()>();
        let (done_tx, done_rx) = channel::<()>();
        let start_rx = Arc::new(Mutex::new(start_rx));

        let producer_queue = Arc::clone(&queue);
        let producer_done_tx = done_tx.clone();
        let producer_start_rx = Arc::clone(&start_rx);
        thread::spawn(move || {
            let input = Input {
                snake_id: 1,
                direction: Direction::Up,
            };
            loop {
                if producer_start_rx.lock().unwrap().recv().is_err() {
                    break;
                }
                for _ in 0..NUM_MESSAGES {
                    while !producer_queue.produce(input) {
                        thread::yield_now();
                    }
                }
                producer_done_tx.send(()).unwrap();
            }
        });

        let consumer_done_tx = done_tx;
        let consumer_start_rx = Arc::clone(&start_rx);
        thread::spawn(move || {
            loop {
                if consumer_start_rx.lock().unwrap().recv().is_err() {
                    break;
                }
                for _ in 0..NUM_MESSAGES {
                    while queue.consume().is_none() {
                        thread::yield_now();
                    }
                    busy_spin(BUSY_SPIN_ITERS); // Simulate work
                }
                consumer_done_tx.send(()).unwrap();
            }
        });

        b.iter(|| {
            start_tx.send(()).unwrap();
            start_tx.send(()).unwrap();
            done_rx.recv().unwrap();
            done_rx.recv().unwrap();
        });
    });

    // --- Consumer is faster than the producer (queue is often empty) ---
    group.bench_function("consumer_faster_producer_slower", |b| {
        let queue = Arc::new(Spsc::<Input, QUEUE_CAPACITY>::new());
        let (start_tx, start_rx) = channel::<()>();
        let (done_tx, done_rx) = channel::<()>();
        let start_rx = Arc::new(Mutex::new(start_rx));

        let producer_queue = Arc::clone(&queue);
        let producer_done_tx = done_tx.clone();
        let producer_start_rx = Arc::clone(&start_rx);
        thread::spawn(move || {
            let input = Input {
                snake_id: 1,
                direction: Direction::Up,
            };
            loop {
                if producer_start_rx.lock().unwrap().recv().is_err() {
                    break;
                }
                for _ in 0..NUM_MESSAGES {
                    while !producer_queue.produce(input) {
                        thread::yield_now();
                    }
                    busy_spin(BUSY_SPIN_ITERS); // Simulate work
                }
                producer_done_tx.send(()).unwrap();
            }
        });

        let consumer_done_tx = done_tx;
        let consumer_start_rx = Arc::clone(&start_rx);
        thread::spawn(move || {
            loop {
                if consumer_start_rx.lock().unwrap().recv().is_err() {
                    break;
                }
                for _ in 0..NUM_MESSAGES {
                    while queue.consume().is_none() {
                        thread::yield_now();
                    }
                }
                consumer_done_tx.send(()).unwrap();
            }
        });

        b.iter(|| {
            start_tx.send(()).unwrap();
            start_tx.send(()).unwrap();
            done_rx.recv().unwrap();
            done_rx.recv().unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    spsc_throughput_bench,
    spsc_latency_bench,
    spsc_contention_bench
);
criterion_main!(benches);
