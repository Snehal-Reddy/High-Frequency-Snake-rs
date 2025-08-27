use crate::game::{
    apple::{Apple, APPLE_CAPACITY},
    grid::{self, Grid},
    snake::{Snake, SNAKE_CAPACITY},
    types::{Input},
};
use grid::Cell;
use std::{collections::HashMap};
use rand::Rng;

pub struct GameState {
    // TODO: static array might give better perf
    pub snakes: HashMap<u32, Snake>,
    // TODO: static array might give better perf
    pub apples: Vec<Apple>,
    pub grid: Grid,
}

impl GameState {
    // clean this shit up
    pub fn random() -> Self {
        let mut random_snakes = HashMap::<u32, Snake>::with_capacity(SNAKE_CAPACITY);
        let mut rng = rand::rng();
        for index in 0..SNAKE_CAPACITY {
            let mut snake = Snake::new(
                index as u32,
                rng.random(),
                rng.random(),
            );
            for _ in 0..3 {
                snake.grow();
            }
            random_snakes.insert(index as u32, snake);
        }

        let mut grid = Grid::new();
        for snake in random_snakes.values() {
            for part in &snake.body {
                grid.set_cell(*part, Cell::Snake);
            }
        }

        let mut random_apples = Vec::<Apple>::with_capacity(APPLE_CAPACITY);
        for _ in 0..APPLE_CAPACITY {
            loop {
                let apple = Apple::new(rng.random());
                if grid.get_cell(&apple.position) == Cell::Empty {
                    grid.set_cell(apple.position, Cell::Apple);
                    random_apples.push(apple);
                    break;
                }
            }
        }

        Self {
            snakes: random_snakes,
            apples: random_apples,
            grid,
        }
    }
    pub fn new() -> Self {
        Self {
            // TODO: static array might give better perf
            snakes: HashMap::with_capacity(SNAKE_CAPACITY),
            // TODO: static array might give better perf
            apples: Vec::with_capacity(APPLE_CAPACITY),
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
            self.snakes.get_mut(&input.snake_id).unwrap().change_direction(input.direction);
        }
        // TODO: Try inlining functions
        for (_id, snake) in self.snakes.iter_mut() {
            snake.move_forward();
            let head = snake.body.front().unwrap();
            if self.grid.get_cell(head) == Cell::Apple {
                snake.grow();
            }
            else if self.grid.get_cell(head) == Cell::Snake {
                snake.is_alive = false;
            }
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
