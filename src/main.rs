mod game;

use game::engine::GameState;
use game::types::{Direction, Input};

fn main() {
    println!("Snake Battle Royale: Low Level Optimization Playground");

    // Initialize game state
    let mut game_state = GameState::new();

    println!("Game state initialized.");

    // Simulate a single tick with some dummy inputs
    let dummy_inputs = vec![
        Input {
            snake_id: 1,
            direction: Direction::Up,
        },
        Input {
            snake_id: 2,
            direction: Direction::Left,
        },
    ];

    println!("\n--- Simulating one tick ---");
    game_state.tick(&dummy_inputs);
    println!("--- Tick simulation finished ---\n");
}
