use criterion::{Criterion, criterion_group, criterion_main};
use high_frequency_snake::game::{
    engine::GameState,
    types::{Direction, Input},
};
use high_frequency_snake::ipc::spsc::Spsc;
use rand::Rng;
use std::hint::black_box;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

const QUEUE_CAPACITY: usize = 1024;
const NUM_TICKS: usize = 1000;
const MIN_SNAKES: usize = 100;
const MAX_SNAKES: usize = 1000;
const SNAKE_STEP: usize = 100;

/// Generate random inputs for a given number of snakes
fn generate_random_inputs(num_snakes: usize, input_ratio: f64) -> Vec<Input> {
    let mut rng = rand::rng();
    let num_inputs = (num_snakes as f64 * input_ratio) as usize;
    
    (0..num_inputs)
        .map(|_| Input {
            snake_id: rng.random_range(0..num_snakes) as u32,
            direction: rng.random(),
        })
        .collect()
}

/// Benchmark the hot path with pinned threads (like main.rs) but measure only the hot path timing
fn hot_path_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path");
    group.throughput(criterion::Throughput::Elements(NUM_TICKS as u64));

    let core_ids = core_affinity::get_core_ids();
    if core_ids.is_none() || core_ids.as_ref().unwrap().len() < 2 {
        println!("Skipping hot path benchmark: at least 2 CPU cores required.");
        return;
    }
    let core_a = core_ids.as_ref().unwrap()[0];
    let core_b = core_ids.as_ref().unwrap()[1];

    for num_snakes in (MIN_SNAKES..=MAX_SNAKES).step_by(SNAKE_STEP) {
        group.bench_function(&format!("{}_snakes", num_snakes), |b| {
            // Setup: Create queue and game state (outside of measurement)
            let queue = Arc::new(Spsc::<Input, QUEUE_CAPACITY>::new());
            let mut game_state = GameState::random();
            while game_state.snakes.len() < num_snakes {
                game_state = GameState::random();
            }
            
            // Pre-fill queue with inputs to simulate continuous operation
            let inputs = generate_random_inputs(num_snakes, 0.25);
            for input in &inputs {
                while !queue.produce(*input) {
                    thread::yield_now();
                }
            }
            
            let mut inputs_buffer = Vec::with_capacity(QUEUE_CAPACITY);
            
            // Measure only the hot path: consume + tick
            b.iter(|| {
                let mut input_count = 0;
                let start = std::time::Instant::now();
                
                // Drain the queue (SPSC consume) - this is the hot path
                while let Some(input) = queue.consume() {
                    inputs_buffer.push(input);
                    input_count += 1;
                }
                
                // Process the collected inputs (game.tick()) - this is the hot path
                if !inputs_buffer.is_empty() {
                    black_box(game_state.tick(&inputs_buffer));
                    inputs_buffer.clear();
                }
                
                let total_duration = start.elapsed();
                // Calculate per-input latency: (consume + tick) per input
                let per_input_duration = if input_count > 0 {
                    total_duration / input_count
                } else {
                    total_duration // Fallback if no inputs
                };
                
                // Re-fill queue for next iteration (simulates continuous input)
                for input in &inputs {
                    while !queue.produce(*input) {
                        thread::yield_now();
                    }
                }
                
                // Return the per-input duration to prevent optimization
                black_box(per_input_duration);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, hot_path_bench);
criterion_main!(benches);
