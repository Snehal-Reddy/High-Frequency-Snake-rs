use crate::game::{
    apple::{APPLE_CAPACITY, Apple},
    grid::{self, Grid},
    snake::{SNAKE_CAPACITY, Snake, GridAwareSnake},
    types::{Input, Point},
};
use grid::Cell;
use rand::Rng;

pub struct GameState {
    // Using wrapper types that automatically manage grid updates
    pub snakes: Vec<GridAwareSnake>,
    pub num_apples: u64,
    pub grid: Grid,
}

impl GameState {
    // clean this shit up
    pub fn random() -> Self {
        let mut random_snakes = Vec::< GridAwareSnake>::with_capacity(SNAKE_CAPACITY);
        let mut grid = Grid::new();
        let mut rng = rand::rng();
        let mut num_apples = 0;

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
                        snake.move_forward(true); // Move forward with growth
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
                    let snake = Snake::new(index as u32, start_pos, rng.random());
                    break snake;
                }
            };

            // Add snake to the game state using wrapper
            let grid_aware_snake = GridAwareSnake::new(snake, &mut grid);
            random_snakes.push(grid_aware_snake);
        }

        // Spawn apples in empty spaces
        for _ in 0..APPLE_CAPACITY {
            let mut attempts = 0;
            loop {
                let apple = Apple::new(rng.random());
                if grid.get_cell(&apple.position) == Cell::Empty {
                    grid.set_cell(apple.position, Cell::Apple);
                    num_apples += 1;
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
            num_apples: num_apples,
            grid,
        }
    }
    
    pub fn new() -> Self {
        Self {
            snakes: Vec::<GridAwareSnake>::with_capacity(SNAKE_CAPACITY),
            num_apples: 0,
            grid: Grid::new(),
        }
    }

    /// The main game loop (hot path baby!)
    pub fn tick(&mut self, inputs: &[Input]) {
        // Process inputs and update snake directions
        // TODO: Wonder if sorting inputs will be faster for cache?
        for input in inputs {
            // Processing dead snakes as well, do not want to add a branch.
            // TODO: Bounds check?
            self.snakes[input.snake_id as usize].change_direction(input.direction);
        }

        let mut consumed_apples = 0;
        
        for snake in self.snakes.iter_mut() {
            // Branch prediction might have a headache with this.
            if !snake.is_alive() {
                continue;
            }

            // Check for apple consumption before moving
            let will_eat_apple = if snake.head().is_some() {
                let new_head = snake.snake().calculate_new_head();
                self.grid.get_cell(&new_head) == Cell::Apple
            } else {
                false
            };
            
            // Move snake (collision detection happens automatically)
            if snake.move_forward(&mut self.grid, will_eat_apple) {
                // If snake was going to eat an apple, handle it now
                if will_eat_apple {
                    if let Some(head) = snake.head().copied() {
                        self.grid.set_cell(head, Cell::Empty);
                        self.num_apples -= 1;
                        consumed_apples += 1;
                        break;
                    }
                }
            }
            // If snake.move_forward() returned false, snake is already dead
        }

        // Spawn new apples to replace consumed ones
        if consumed_apples > 0 {
            for _ in 0..consumed_apples {
                self.spawn_apple();
            }
        }
    }

    /// Add an apple to the game state (grid update happens automatically)
    pub fn add_apple(&mut self, apple: Apple) {
        if self.num_apples < APPLE_CAPACITY as u64 {
            self.grid.set_cell(apple.position, Cell::Apple);
            self.num_apples += 1;
        }
    }

    /// Spawn a new apple at a random empty position
    fn spawn_apple(&mut self) {
        if self.num_apples >= APPLE_CAPACITY as u64 {
            return; // Don't spawn if at capacity
        }

        let mut rng = rand::rng();
        for _attempts in 0..100 {
            // Limit attempts to avoid infinite loop
            let position = rng.random::<Point>();
            if self.grid.get_cell(&position) == Cell::Empty {
                self.grid.set_cell(position, Cell::Apple);
                self.num_apples += 1;
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
