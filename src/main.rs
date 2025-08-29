use high_frequency_snake::game::engine::GameState;
use high_frequency_snake::game::types::{Direction, Input};
use high_frequency_snake::ipc::spsc::Spsc;
use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

const QUEUE_CAPACITY: usize = 1024;

#[cfg(feature = "profile")]
fn get_cpu_cycles() -> u64 {
    unsafe { std::arch::x86_64::_rdtsc() }
}

fn main() {
    println!("Snake Battle Royale: Low Level Optimization Playground");

    // Get the available CPU cores
    let core_ids = core_affinity::get_core_ids().unwrap();
    if core_ids.len() < 2 {
        panic!("This application requires at least 2 CPU cores.");
    }

    // Create a shared SPSC queue
    let queue = Arc::new(Spsc::<Input, QUEUE_CAPACITY>::new());
    let producer_queue = Arc::clone(&queue);
    let consumer_queue = Arc::clone(&queue);

    // --- Input Generator Thread ---
    let input_thread_core = core_ids[0];
    let input_generator = thread::spawn(move || {
        // Pin this thread to the first core
        core_affinity::set_for_current(input_thread_core);

        let mut rng = rand::rng();
        println!(
            "Input generator thread started on core {:?}",
            input_thread_core.id
        );

        loop {
            let input = Input {
                snake_id: rng.random_range(1..=1000), // Simulate for 1000 snakes
                direction: rng.random(),
            };

            // Push to the queue
            while !producer_queue.produce(input) {
                // Queue is full, spin for a moment
                thread::yield_now();
            }
        }
    });

    // --- Game Logic Thread ---
    let game_thread_core = core_ids[1];
    let game_logic = thread::spawn(move || {
        // Pin this thread to the second core
        core_affinity::set_for_current(game_thread_core);

        let mut game_state = GameState::random();
        let mut inputs = Vec::with_capacity(QUEUE_CAPACITY);
        println!(
            "Game logic thread started on core {:?}",
            game_thread_core.id
        );

        #[cfg(feature = "profile")]
        {
            let mut tick_count = 0u64;
            let mut total_consume_cycles = 0u64;
            let mut total_tick_cycles = 0u64;
            let mut min_consume_cycles = u64::MAX;
            let mut max_consume_cycles = 0u64;
            let mut min_tick_cycles = u64::MAX;
            let mut max_tick_cycles = 0u64;
            let start_time = Instant::now();

            loop {
                // Measure the consume part
                let consume_start_cycles = get_cpu_cycles();
                while let Some(input) = consumer_queue.consume() {
                    inputs.push(input);
                }
                let consume_end_cycles = get_cpu_cycles();
                let consume_cycles = consume_end_cycles - consume_start_cycles;

                // Process the collected inputs
                if !inputs.is_empty() {
                    // Measure the tick part
                    let tick_start_cycles = get_cpu_cycles();
                    game_state.tick(&inputs);
                    let tick_end_cycles = get_cpu_cycles();
                    let tick_cycles = tick_end_cycles - tick_start_cycles;
                    
                    // Update consume statistics
                    total_consume_cycles += consume_cycles;
                    min_consume_cycles = min_consume_cycles.min(consume_cycles);
                    max_consume_cycles = max_consume_cycles.max(consume_cycles);
                    
                    // Update tick statistics
                    total_tick_cycles += tick_cycles;
                    min_tick_cycles = min_tick_cycles.min(tick_cycles);
                    max_tick_cycles = max_tick_cycles.max(tick_cycles);
                    
                    inputs.clear();
                }

                tick_count += 1;

                // Report performance every 1000 ticks
                if tick_count % 1000 == 0 {
                    let elapsed = start_time.elapsed();
                    let ticks_per_second = tick_count as f64 / elapsed.as_secs_f64();
                    let avg_consume_cycles = total_consume_cycles / tick_count;
                    let avg_tick_cycles = total_tick_cycles / tick_count;
                    
                    println!(
                        "Tick {}: {:.2} ticks/sec | Consume: avg={} cycles, min={} cycles, max={} cycles | Tick: avg={} cycles, min={} cycles, max={} cycles",
                        tick_count, ticks_per_second, 
                        avg_consume_cycles, min_consume_cycles, max_consume_cycles,
                        avg_tick_cycles, min_tick_cycles, max_tick_cycles
                    );
                }
            }
        }

        #[cfg(not(feature = "profile"))]
        {
            loop {
                // Drain the queue
                while let Some(input) = consumer_queue.consume() {
                    inputs.push(input);
                }

                // Process the collected inputs
                if !inputs.is_empty() {
                    game_state.tick(&inputs);
                    inputs.clear();
                }
            }
        }
    });

    input_generator.join().unwrap();
    game_logic.join().unwrap();
}
