use crate::game::grid::{GRID_HEIGHT, GRID_WIDTH, Cell, Grid};
use crate::game::types::{Direction, Point};
use std::collections::VecDeque;

pub const SNAKE_CAPACITY: usize = 1024;

pub struct Snake {
    pub id: u32,
    // TODO: Static array might give better perf
    pub body: VecDeque<Point>,
    pub direction: Direction,
    pub is_alive: bool,
}

impl Snake {
    pub fn new(id: u32, start_pos: Point, initial_direction: Direction) -> Self {
        let mut body = VecDeque::new();
        body.push_front(start_pos);
        Self {
            id,
            body,
            direction: initial_direction,
            is_alive: true,
        }
    }

    pub fn move_forward(&mut self) {
        let new_head = self.calculate_new_head();
        self.body.push_front(new_head);
        self.body.pop_back();
    }
    
    /// Calculate where the snake's head will be after moving forward
    pub fn calculate_new_head(&self) -> Point {
        let current_head = self.body.front().unwrap();
        match self.direction {
            Direction::Up => Point {
                x: current_head.x,
                y: if current_head.y == 0 {
                    (GRID_HEIGHT - 1) as u16
                } else {
                    current_head.y - 1
                },
            },
            Direction::Down => Point {
                x: current_head.x,
                y: if current_head.y == (GRID_HEIGHT - 1) as u16 {
                    0
                } else {
                    current_head.y + 1
                },
            },
            Direction::Left => Point {
                x: if current_head.x == 0 {
                    (GRID_WIDTH - 1) as u16
                } else {
                    current_head.x - 1
                },
                y: current_head.y,
            },
            Direction::Right => Point {
                x: if current_head.x == (GRID_WIDTH - 1) as u16 {
                    0
                } else {
                    current_head.x + 1
                },
                y: current_head.y,
            },
        }
    }

    pub fn grow(&mut self) {
        let tail = *self.body.back().unwrap();
        self.body.push_back(tail);
    }

    pub fn change_direction(&mut self, new_direction: Direction) {
        // Prevent snake from reversing on itself
        let can_change = !matches!(
            (self.direction, new_direction),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        );
        if can_change {
            self.direction = new_direction;
        }
    }
}

/// Smart wrapper around Snake that automatically manages grid updates
pub struct GridAwareSnake {
    snake: Snake,
}

impl GridAwareSnake {
    /// Create a new GridAwareSnake. The snake will be added to the grid immediately.
    pub fn new(snake: Snake, grid: &mut Grid) -> Self {
        let mut wrapper = Self { snake };
        
        // Add initial snake body to grid
        wrapper.update_grid_with_body(grid);
        
        wrapper
    }
    
    /// Move the snake forward, automatically updating the grid
    /// Returns true if movement was successful, false if collision occurred
    pub fn move_forward(&mut self, grid: &mut Grid) -> bool {
        // Calculate new head position
        let new_head = self.snake.calculate_new_head();
        
        // Check for collisions BEFORE moving
        if grid.get_cell(&new_head) == Cell::Snake {
            // Collision detected - mark as dead
            self.snake.is_alive = false;
            return false;
        }
        
        // Check for self-collision (head hitting own body)
        let snake_body = &self.snake.body;
        let head_in_body = snake_body.iter().skip(1).any(|part| *part == new_head);
        if head_in_body {
            // Self-collision detected - mark as dead
            self.snake.is_alive = false;
            return false;
        }
        
        // No collision - proceed with movement
        // Clear the tail position from grid before moving
        if let Some(tail) = self.snake.body.back() {
            grid.set_cell(*tail, Cell::Empty);
        }
        
        // Move the snake
        self.snake.move_forward();
        
        // Update grid with new head position
        if let Some(head) = self.snake.body.front() {
            grid.set_cell(*head, Cell::Snake);
        }
        
        return true;
    }
    
    /// Grow the snake, automatically updating the grid
    pub fn grow(&mut self, grid: &mut Grid) {
        self.snake.grow();
        
        // Add the new tail segment to grid
        if let Some(new_tail) = self.snake.body.back() {
            grid.set_cell(*new_tail, Cell::Snake);
        }
    }
    
    /// Mark the snake as dead and clear it from the grid
    pub fn die(&mut self, grid: &mut Grid) {
        self.snake.is_alive = false;
        self.clear_from_grid(grid);
    }
    
    /// Change direction (no grid update needed)
    pub fn change_direction(&mut self, new_direction: Direction) {
        self.snake.change_direction(new_direction);
    }
    
    /// Get a reference to the underlying snake
    pub fn snake(&self) -> &Snake {
        &self.snake
    }
    
    /// Get a mutable reference to the underlying snake
    pub fn snake_mut(&mut self) -> &mut Snake {
        &mut self.snake
    }
    
    /// Check if snake is alive
    pub fn is_alive(&self) -> bool {
        self.snake.is_alive
    }
    
    /// Get snake ID
    pub fn id(&self) -> u32 {
        self.snake.id
    }
    
    /// Get snake head position
    pub fn head(&self) -> Option<&Point> {
        self.snake.body.front()
    }
    

    
    /// Get snake body
    pub fn body(&self) -> &VecDeque<Point> {
        &self.snake.body
    }
    
    // Private helper methods
    
    fn update_grid_with_body(&self, grid: &mut Grid) {
        for part in &self.snake.body {
            grid.set_cell(*part, Cell::Snake);
        }
    }
    
    fn clear_from_grid(&self, grid: &mut Grid) {
        for part in &self.snake.body {
            grid.set_cell(*part, Cell::Empty);
        }
    }
}
