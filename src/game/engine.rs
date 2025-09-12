use crate::game::{
    apple::{APPLE_CAPACITY, Apple},
    grid::{self, Grid},
    snake::{SNAKE_CAPACITY, Snake, GridAwareSnake},
    types::{Input, Point},
};
use grid::Cell;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct MovementRecord {
    pub snake_id: u32,
    pub new_head: Point,
    pub cell_at_new_head: Cell,
}

pub struct GameState {
    // Using wrapper types that automatically manage grid updates
    pub snakes: Vec<GridAwareSnake>,
    pub num_apples: u64,
    pub grid: Grid,
    // Pre-allocated movement records to avoid allocation on every tick
    pub movement_records: Vec<MovementRecord>,
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
            movement_records: vec![MovementRecord {
                snake_id: 0,
                new_head: Point { x: 0, y: 0 },
                cell_at_new_head: Cell::Empty,
            }; SNAKE_CAPACITY],
        }
    }
    
    pub fn new() -> Self {
        Self {
            snakes: Vec::<GridAwareSnake>::with_capacity(SNAKE_CAPACITY),
            num_apples: 0,
            grid: Grid::new(),
            movement_records: vec![MovementRecord {
                snake_id: 0,
                new_head: Point { x: 0, y: 0 },
                cell_at_new_head: Cell::Empty,
            }; SNAKE_CAPACITY],
        }
    }

    /// The legacy game loop (pre cache-aware)
    pub fn tick_legacy(&mut self, inputs: &[Input]) {
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

    /// The main game loop (cache-aware)
    pub fn tick(&mut self, inputs: &[Input]) {
        // Phase 1: Process inputs (unchanged)
        for input in inputs {
            self.snakes[input.snake_id as usize].change_direction(input.direction);
        }

        // Phase 1: Collect Records (NO Grid Reads)
        let mut record_count = 0;
        for snake in &self.snakes {
            if !snake.is_alive() { continue; }

            let new_head = snake.calculate_new_head();

            // Direct index assignment - we know the capacity is SNAKE_CAPACITY
            self.movement_records[record_count] = MovementRecord {
                snake_id: snake.id(),
                new_head,
                cell_at_new_head: Cell::Empty, // Will be filled in Phase 3
            };
            record_count += 1;
        }

        // Phase 2: Sort by Spatial Locality (only alive snakes)
        self.movement_records[..record_count].sort_by_key(|record: &MovementRecord| (record.new_head.y, record.new_head.x));

        // Phase 3-5: Combined Loop (Read, Process, Write Immediately)
        let mut consumed_apples: u64 = 0;
        let mut previous_new_head: Option<Point> = None;

        for record in &mut self.movement_records[..record_count] {
            // Phase 3: Read cell value (cache-friendly since records are sorted)
            record.cell_at_new_head = self.grid.get_cell(&record.new_head);

            if record.cell_at_new_head == Cell::Snake {
                self.snakes[record.snake_id as usize].mark_dead();
                continue; // Skip this snake
            }

            if let Some(prev_pos) = previous_new_head {
                if record.new_head == prev_pos {
                    self.snakes[record.snake_id as usize].mark_dead();
                    continue; // Skip this snake
                }
            }

            previous_new_head = Some(record.new_head);

            let will_grow = record.cell_at_new_head == Cell::Apple;
            if will_grow {
                consumed_apples += 1;
            }

            // Write new head
            self.grid.set_cell(record.new_head, Cell::Snake);

            // Write tail clearing (if not growing) - get tail position directly from snake
            if !will_grow {
                let snake = &self.snakes[record.snake_id as usize];
                if let Some(tail_pos) = snake.tail_position() {
                    self.grid.set_cell(tail_pos, Cell::Empty);
                }
            }

            // Update snake body (no grid access)
            self.snakes[record.snake_id as usize].update_body(will_grow);
        }

        // Phase 6: Spawn new apples to replace consumed ones
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
