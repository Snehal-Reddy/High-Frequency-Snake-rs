use high_frequency_snake::game::engine::GameState;
use high_frequency_snake::game::types::{Direction, Input};
use high_frequency_snake::ipc::spsc::Spsc;
use rand::Rng;
use std::sync::Arc;
use std::thread;

const QUEUE_CAPACITY: usize = 1024;

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
        println!("Input generator thread started on core {:?}", input_thread_core.id);

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
        println!("Game logic thread started on core {:?}", game_thread_core.id);

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
    });

    input_generator.join().unwrap();
    game_logic.join().unwrap();
}
