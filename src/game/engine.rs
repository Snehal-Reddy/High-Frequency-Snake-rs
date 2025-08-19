use crate::game::{
    apple::Apple,
    grid::Grid,
    snake::Snake,
    types::{Direction, Input, Point},
};
use std::collections::HashMap;

pub struct GameState {
    // TODO: static array might give better perf
    pub snakes: HashMap<u32, Snake>,
    // TODO: static array might give better perf
    pub apples: Vec<Apple>,
    pub grid: Grid,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            // TODO: static array might give better perf
            snakes: HashMap::new(),
            // TODO: static array might give better perf
            apples: Vec::new(),
            grid: Grid::new(),
        }
    }

    /// The main game loop.
    /// In this dummy implementation, it only prints received inputs.
    pub fn tick(&mut self, inputs: &[Input]) {
        for input in inputs {
            println!(
                "Tick: Received input for snake {}: {:?}. Game state not changed.",
                input.snake_id, input.direction
            );
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
