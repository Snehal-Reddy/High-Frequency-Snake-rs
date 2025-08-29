use crate::game::{
    apple::{APPLE_CAPACITY, Apple},
    grid::{self, GRID_HEIGHT, GRID_WIDTH, Grid},
    snake::{SNAKE_CAPACITY, Snake},
    types::{Input, Point},
};
use grid::Cell;
use rand::Rng;
use std::collections::HashMap;

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
        let mut grid = Grid::new();
        let mut rng = rand::rng();

        // Spawn snakes with collision detection
        for index in 0..SNAKE_CAPACITY {
            let mut attempts = 0;
            let snake = loop {
                let start_pos = rng.random::<Point>();

                // Check if the starting position is empty
                if grid.get_cell(&start_pos) == Cell::Empty {
                    let mut snake = Snake::new(index as u32, start_pos, rng.random());

                    // Grow the snake and check each new segment
                    let mut valid_growth = true;
                    for _ in 0..3 {
                        snake.grow();
                        // Check if the new tail position is valid
                        if let Some(tail) = snake.body.back() {
                            if grid.get_cell(tail) != Cell::Empty {
                                valid_growth = false;
                                break;
                            }
                        }
                    }

                    if valid_growth {
                        break snake;
                    }
                }

                attempts += 1;
                if attempts > 1000 {
                    // Fallback: create a minimal snake if we can't find space
                    let start_pos = Point { x: 0, y: 0 };
                    let mut snake = Snake::new(index as u32, start_pos, rng.random());
                    break snake;
                }
            };

            // Add snake to the game state
            random_snakes.insert(index as u32, snake);

            // Update grid with snake positions
            if let Some(snake) = random_snakes.get(&(index as u32)) {
                for part in &snake.body {
                    grid.set_cell(*part, Cell::Snake);
                }
            }
        }

        // Spawn apples in empty spaces
        let mut random_apples = Vec::<Apple>::with_capacity(APPLE_CAPACITY);
        for _ in 0..APPLE_CAPACITY {
            let mut attempts = 0;
            loop {
                let apple = Apple::new(rng.random());
                if grid.get_cell(&apple.position) == Cell::Empty {
                    grid.set_cell(apple.position, Cell::Apple);
                    random_apples.push(apple);
                    break;
                }
                attempts += 1;
                if attempts > 1000 {
                    // Skip this apple if we can't find space
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
    pub fn tick(&mut self, inputs: &[Input]) {
        // Process inputs and update snake directions
        // TODO: Try inlining functions
        for input in inputs {
            if let Some(snake) = self.snakes.get_mut(&input.snake_id) {
                snake.change_direction(input.direction);
            }
        }

        // Clear the grid for the new tick
        self.grid = Grid::new();

        // Update grid with apple positions first
        for apple in &self.apples {
            self.grid.set_cell(apple.position, Cell::Apple);
        }

        // Move snakes and handle collisions
        let mut consumed_apples = 0;
        for (_id, snake) in self.snakes.iter_mut() {
            snake.move_forward();
            let head = snake.body.front().unwrap();

            // Check for snake collisions first
            if self.grid.get_cell(head) == Cell::Snake {
                snake.is_alive = false;
                continue;
            }

            // Check for apple consumption
            if self.grid.get_cell(head) == Cell::Apple {
                snake.grow();
                consumed_apples += 1;
            }

            // Update grid with snake positions
            for part in &snake.body {
                self.grid.set_cell(*part, Cell::Snake);
            }
        }

        // Remove consumed apples and spawn new ones
        if consumed_apples > 0 {
            // Remove consumed apples (any apple that's now under a snake)
            self.apples
                .retain(|apple| self.grid.get_cell(&apple.position) != Cell::Snake);

            // Spawn new apples
            for _ in 0..consumed_apples {
                self.spawn_apple();
            }
        }

        // Cleanup dead snakes
        self.snakes.retain(|_id, snake| {
            if !snake.is_alive {
                // Clear all grid cells occupied by the dead snake
                for part in &snake.body {
                    self.grid.set_cell(*part, Cell::Empty);
                }
                false // Remove from HashMap
            } else {
                true // Keep in HashMap
            }
        });
    }

    /// Spawn a new apple at a random empty position
    fn spawn_apple(&mut self) {
        if self.apples.len() >= APPLE_CAPACITY {
            return; // Don't spawn if at capacity
        }

        let mut rng = rand::rng();
        for _attempts in 0..100 {
            // Limit attempts to avoid infinite loop
            let position = rng.random::<Point>();
            if self.grid.get_cell(&position) == Cell::Empty {
                let apple = Apple::new(position);
                self.apples.push(apple);
                self.grid.set_cell(position, Cell::Apple);
                break;
            }
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
