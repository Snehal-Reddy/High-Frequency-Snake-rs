use crate::game::{
    apple::{APPLE_CAPACITY, Apple, GridAwareApple},
    engine::GameState,
    grid::{self, GRID_HEIGHT, GRID_WIDTH, Grid},
    snake::{SNAKE_CAPACITY, Snake, GridAwareSnake},
    types::{Direction, Point},
};
use grid::Cell;
use rand::Rng;

#[derive(Clone, Copy)]
pub struct DeterministicConfig {
    pub seed: u64,
    pub layout_pattern: LayoutPattern,
    pub initial_snake_length: usize,
}

#[derive(Clone, Copy)]
pub enum LayoutPattern {
    Grid,
    Concentric,
}

impl Default for DeterministicConfig {
    fn default() -> Self {
        Self {
            seed: 42, // Default seed for reproducibility
            layout_pattern: LayoutPattern::Grid,
            initial_snake_length: 3,
        }
    }
}

pub struct DeterministicGenerator;

impl DeterministicGenerator {
    pub fn generate(num_snakes: usize, config: DeterministicConfig) -> GameState {
        let mut grid = Grid::new();
        let mut snakes = Vec::<GridAwareSnake>::with_capacity(num_snakes);
        let mut apples = Vec::new();
        
        // Calculate spacing based on snake count and grid size
        let spacing = Self::calculate_snake_spacing(num_snakes);
        
        // Get positions based on layout pattern
        let snake_positions = match config.layout_pattern {
            LayoutPattern::Grid => Self::calculate_grid_positions(num_snakes, spacing),
            LayoutPattern::Concentric => Self::calculate_concentric_positions(num_snakes),
        };
        
        // Place snakes
        for (i, pos) in snake_positions.iter().enumerate() {
            let mut snake = Snake::new(i as u32, *pos, Direction::Right);
            // Grow to initial length
            for _ in 0..config.initial_snake_length - 1 {
                snake.grow();
            }
            let grid_aware_snake = GridAwareSnake::new(snake, &mut grid);
            snakes.push(grid_aware_snake);
        }
        
        // Place apples in remaining spaces
        let apple_positions = Self::calculate_apple_positions(&grid, config.seed);
        for pos in apple_positions {
            let apple = Apple::new(pos);
            let grid_aware_apple = GridAwareApple::new(apple, &mut grid);
            apples.push(grid_aware_apple);
        }
        
        GameState {
            snakes,
            apples,
            grid,
        }
    }
    
    /// Generate a deterministic game state with predictable outcomes
    /// This creates a configuration where:
    /// - 25% of snakes will die (collide with each other)
    /// - 25% of snakes will grow (consume apples)
    /// - 50% of snakes will remain unchanged (safe movement)
    pub fn generate_predictable_outcomes(num_snakes: usize, config: DeterministicConfig) -> GameState {
        let mut grid = Grid::new();
        let mut snakes = Vec::<GridAwareSnake>::with_capacity(num_snakes);
        let mut apples = Vec::new();
        
        // Calculate group sizes
        let death_group_size = num_snakes / 4; // 25%
        let apple_group_size = num_snakes / 4; // 25%
        let safe_group_size = num_snakes - death_group_size - apple_group_size; // 50%
        
        // Place death group snakes (will collide) - close together
        let death_start_x = 100;
        let death_start_y = 100;
        for i in 0..death_group_size {
            // Place snakes in pairs that will collide
            let (x, y, initial_direction) = if i % 2 == 0 {
                // Even snakes start on the left, moving right
                (death_start_x, death_start_y + (i / 2) * 10, Direction::Right)
            } else {
                // Odd snakes start on the right, moving left
                (death_start_x + 10, death_start_y + (i / 2) * 10, Direction::Left)
            };
            
            let pos = Point { x: x as u16, y: y as u16 };
            let mut snake = Snake::new(i as u32, pos, initial_direction);
            for _ in 0..config.initial_snake_length - 1 {
                snake.grow();
            }
            let grid_aware_snake = GridAwareSnake::new(snake, &mut grid);
            snakes.push(grid_aware_snake);
        }
        
        // Place apples first, then place apple group snakes next to them
        let apple_start_x = 200;
        let apple_start_y = 100;
        let apples_to_place = apple_group_size.min(APPLE_CAPACITY);
        
        // Place apples
        for i in 0..apples_to_place {
            let apple_x = apple_start_x + (i % 10) * 20;
            let apple_y = apple_start_y + (i / 10) * 20;
            let apple_pos = Point { x: apple_x as u16, y: apple_y as u16 };
            let apple = Apple::new(apple_pos);
            let grid_aware_apple = GridAwareApple::new(apple, &mut grid);
            apples.push(grid_aware_apple);
        }
        
        // Place apple group snakes
        for i in 0..apple_group_size {
            let snake_x = if i < apples_to_place {
                // Place next to apple if apple exists
                let apple_x = apple_start_x + (i % 10) * 20;
                apple_x.saturating_sub(1)
            } else {
                // Place in apple area even if no apple
                apple_start_x + (i % 10) * 20 + 5
            };
            let snake_y = apple_start_y + (i / 10) * 20;
            let snake_pos = Point { x: snake_x as u16, y: snake_y as u16 };
            
            let idx = i + death_group_size;
            let mut snake = Snake::new(idx as u32, snake_pos, Direction::Right);
            for _ in 0..config.initial_snake_length - 1 {
                snake.grow();
            }
            let grid_aware_snake = GridAwareSnake::new(snake, &mut grid);
            snakes.push(grid_aware_snake);
        }
        
        // Place safe group snakes (will survive) - far from others
        let safe_start_x = 500;
        let safe_start_y = 100;
        for i in 0..safe_group_size {
            let x = safe_start_x + (i % 20) * 50; // Wide spacing
            let y = safe_start_y + (i / 20) * 50;
            let pos = Point { x: x as u16, y: y as u16 };
            let idx = i + death_group_size + apple_group_size;
            let mut snake = Snake::new(idx as u32, pos, Direction::Right);
            for _ in 0..config.initial_snake_length - 1 {
                snake.grow();
            }
            let grid_aware_snake = GridAwareSnake::new(snake, &mut grid);
            snakes.push(grid_aware_snake);
        }
        
        // Add some additional random apples if we have capacity
        if apples.len() < APPLE_CAPACITY {
            let additional_apple_positions = Self::calculate_apple_positions(&grid, config.seed);
            for pos in additional_apple_positions.iter().take(APPLE_CAPACITY - apples.len()) {
                let apple = Apple::new(*pos);
                let grid_aware_apple = GridAwareApple::new(apple, &mut grid);
                apples.push(grid_aware_apple);
            }
        }
        
        GameState {
            snakes,
            apples,
            grid,
        }
    }
    
    /// Validate that the generated game state is reasonable
    pub fn validate_game_state(game_state: &GameState, expected_snakes: usize) -> bool {
        // Check if we have the expected number of snakes
        if game_state.snakes.len() != expected_snakes {
            println!("❌ Expected {} snakes, got {}", expected_snakes, game_state.snakes.len());
            return false;
        }
        
        // Check if snakes are reasonably spaced (not all clumped together)
        let positions: Vec<_> = game_state.snakes.iter()
            .map(|s| s.head().copied().unwrap_or(Point { x: 0, y: 0 }))
            .collect();
        
        // Check minimum distance between any two snakes
        let mut min_distance = u32::MAX;
        for i in 0..positions.len() {
            for j in i+1..positions.len() {
                let dx = (positions[i].x as i32 - positions[j].x as i32).abs() as u32;
                let dy = (positions[i].y as i32 - positions[j].y as i32).abs() as u32;
                let distance = dx + dy; // Manhattan distance
                min_distance = min_distance.min(distance);
            }
        }
        
        // Snakes should be at least 2 cells apart
        if min_distance < 2 {
            println!("❌ Snakes too close together: minimum distance = {}", min_distance);
            return false;
        }
        
        // Check if we have reasonable number of apples
        let active_apples = game_state.apples.iter().filter(|a| a.is_spawned()).count();
        if active_apples == 0 {
            println!("❌ No active apples in game state");
            return false;
        }
        
        if active_apples > APPLE_CAPACITY {
            println!("❌ Too many active apples: {} > {}", active_apples, APPLE_CAPACITY);
            return false;
        }
        
        println!("✅ Valid game state: {} snakes, {} apples, min distance = {}", 
                game_state.snakes.len(), active_apples, min_distance);
        true
    }
    
    fn calculate_snake_spacing(num_snakes: usize) -> usize {
        // For 4000x4000 grid = 16,000,000 total cells
        // If we want snakes to be reasonably spaced:
        let total_cells = GRID_WIDTH * GRID_HEIGHT;
        let available_cells = total_cells / 2; // Leave space for apples and snake bodies
        let spacing = (available_cells as f64 / num_snakes as f64).sqrt() as usize;
        spacing.max(2) // Minimum 2 cells between snakes
    }
    
    fn calculate_grid_positions(num_snakes: usize, spacing: usize) -> Vec<Point> {
        let mut positions = Vec::new();
        let mut x = spacing;
        let mut y = spacing;
        
        for _ in 0..num_snakes {
            if x >= GRID_WIDTH - spacing {
                x = spacing;
                y += spacing;
            }
            if y >= GRID_HEIGHT - spacing {
                break; // Grid is full
            }
            positions.push(Point { x: x as u16, y: y as u16 });
            x += spacing;
        }
        positions
    }
    
    fn calculate_concentric_positions(num_snakes: usize) -> Vec<Point> {
        let mut positions = Vec::new();
        let center = Point { 
            x: (GRID_WIDTH / 2) as u16, 
            y: (GRID_HEIGHT / 2) as u16 
        };
        let mut radius = 2;
        let mut angle_step = 2.0 * std::f64::consts::PI / num_snakes as f64;
        
        for i in 0..num_snakes {
            let angle = i as f64 * angle_step;
            let x = center.x + (radius as f64 * angle.cos()) as u16;
            let y = center.y + (radius as f64 * angle.sin()) as u16;
            
            if x < GRID_WIDTH as u16 && y < GRID_HEIGHT as u16 {
                positions.push(Point { x, y });
            } else {
                radius += 2; // Increase radius if we hit boundaries
                // Recalculate this position
                if i > 0 {
                    // Try again with larger radius
                    let x = center.x + (radius as f64 * angle.cos()) as u16;
                    let y = center.y + (radius as f64 * angle.sin()) as u16;
                    if x < GRID_WIDTH as u16 && y < GRID_HEIGHT as u16 {
                        positions.push(Point { x, y });
                    }
                }
            }
        }
        positions
    }
    
    fn calculate_apple_positions(grid: &Grid, _seed: u64) -> Vec<Point> {
        let mut positions = Vec::new();
        // TODO: Use seeded RNG for true determinism
        
        // Calculate how many apples we want (reasonable ratio to empty space)
        let empty_cells = GRID_WIDTH * GRID_HEIGHT - 100; // Approximate empty cells after snakes
        let target_apples = (empty_cells / 1000).min(APPLE_CAPACITY); // 1 apple per 1000 empty cells, max 128
        
        // Place apples with reasonable spacing
        let mut count = 0;
        let mut apple_count = 0;
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let pos = Point { x: x as u16, y: y as u16 };
                if grid.get_cell(&pos) == Cell::Empty {
                    if count % 1000 == 0 && apple_count < target_apples { // Every 1000th empty cell
                        positions.push(pos);
                        apple_count += 1;
                    }
                    count += 1;
                }
            }
        }
        
        // If we didn't get enough apples, add more with larger spacing
        if apple_count < target_apples {
            count = 0;
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let pos = Point { x: x as u16, y: y as u16 };
                    if grid.get_cell(&pos) == Cell::Empty {
                        if count % 500 == 0 && apple_count < target_apples { // Every 500th empty cell
                            if !positions.contains(&pos) {
                                positions.push(pos);
                                apple_count += 1;
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
        
        positions
    }
    
    fn calculate_strategic_apple_positions(grid: &Grid, _seed: u64, snakes: &Vec<GridAwareSnake>) -> Vec<Point> {
        let mut positions = Vec::new();
        // TODO: Use seeded RNG for true determinism
        
        // Calculate how many apples we want
        let empty_cells = GRID_WIDTH * GRID_HEIGHT - snakes.len() * 3; // Approximate empty cells after snakes
        let target_apples = (empty_cells / 1000).min(APPLE_CAPACITY); // 1 apple per 1000 empty cells, max 128
        
        // Place apples strategically near apple group snakes
        let mut apple_count = 0;
        let mut count = 0;
        
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let pos = Point { x: x as u16, y: y as u16 };
                if grid.get_cell(&pos) == Cell::Empty {
                    if count % 1000 == 0 && apple_count < target_apples { // Every 1000th empty cell
                        positions.push(pos);
                        apple_count += 1;
                    }
                    count += 1;
                }
            }
        }
        
        // If we didn't get enough apples, add more with larger spacing
        if apple_count < target_apples {
            count = 0;
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let pos = Point { x: x as u16, y: y as u16 };
                    if grid.get_cell(&pos) == Cell::Empty {
                        if count % 500 == 0 && apple_count < target_apples { // Every 500th empty cell
                            if !positions.contains(&pos) {
                                positions.push(pos);
                                apple_count += 1;
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
        
        positions
    }
}

pub struct RandomGenerator;

impl RandomGenerator {
    pub fn generate() -> GameState {
        let mut random_snakes = Vec::<GridAwareSnake>::with_capacity(SNAKE_CAPACITY);
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

        GameState {
            snakes: random_snakes,
            apples: random_apples,
            grid,
        }
    }
}
