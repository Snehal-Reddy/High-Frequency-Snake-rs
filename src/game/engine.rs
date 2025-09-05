use crate::game::{
    apple::{APPLE_CAPACITY, Apple, GridAwareApple},
    grid::{self, Grid},
    snake::{SNAKE_CAPACITY, Snake, GridAwareSnake},
    types::{Input, Point},
};
use grid::Cell;
use rand::Rng;

pub struct GameState {
    // Using wrapper types that automatically manage grid updates
    pub snakes: Vec<GridAwareSnake>,
    pub apples: Vec<GridAwareApple>,
    pub grid: Grid,
}

impl GameState {
    // clean this shit up
    pub fn random() -> Self {
        let mut random_snakes = Vec::< GridAwareSnake>::with_capacity(SNAKE_CAPACITY);
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

            // Add snake to the game state using wrapper
            let grid_aware_snake = GridAwareSnake::new(snake, &mut grid);
            random_snakes.push(grid_aware_snake);
        }

        // Spawn apples in empty spaces
        let mut random_apples = Vec::<GridAwareApple>::with_capacity(APPLE_CAPACITY);
        for _ in 0..APPLE_CAPACITY {
            let mut attempts = 0;
            loop {
                let apple = Apple::new(rng.random());
                if grid.get_cell(&apple.position) == Cell::Empty {
                    let grid_aware_apple = GridAwareApple::new(apple, &mut grid);
                    random_apples.push(grid_aware_apple);
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
            snakes: Vec::<GridAwareSnake>::with_capacity(SNAKE_CAPACITY),
            apples: Vec::<GridAwareApple>::with_capacity(APPLE_CAPACITY),
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
            if snake.move_forward(&mut self.grid) {
                // If snake was going to eat an apple, handle it now
                if will_eat_apple {
                    if let Some(head) = snake.head().copied() {
                        // TODO: Maybe we can optimise this? Seems excessive but 128 so fine for perf now.
                        for apple in &mut self.apples {
                            if apple.position() == head && apple.is_spawned() {
                                snake.grow(&mut self.grid); // Grid update happens automatically
                                apple.consume(&mut self.grid); // Grid update happens automatically
                                consumed_apples += 1;
                                break;
                            }
                        }
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
        if self.apples.len() < APPLE_CAPACITY {
            let grid_aware_apple = GridAwareApple::new(apple, &mut self.grid);
            self.apples.push(grid_aware_apple);
        }
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
                let grid_aware_apple = GridAwareApple::new(apple, &mut self.grid);
                self.apples.push(grid_aware_apple);
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
