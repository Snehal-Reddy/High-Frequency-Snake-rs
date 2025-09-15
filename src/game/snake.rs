use crate::game::grid::{GRID_HEIGHT, GRID_WIDTH, Cell, Grid};
use crate::game::types::{Direction, Point};
use crossbeam_utils::CachePadded;
use tinydeque::TinyDeque;

pub const SNAKE_CAPACITY: usize = 1024;

pub struct Snake {
    pub id: u32,
    pub body: TinyDeque<[Point; 16]>,  // Stack-allocated for small snakes, heap for large
    pub direction: Direction,
    pub is_alive: bool,
}

impl Snake {
    pub fn new(id: u32, start_pos: Point, initial_direction: Direction) -> Self {
        let mut body = TinyDeque::new();
        body.push_front(start_pos);
        Self {
            id,
            body,
            direction: initial_direction,
            is_alive: true,
        }
    }

    pub fn move_forward(&mut self, will_grow: bool) {
        let new_head = self.calculate_new_head();
        self.body.push_front(new_head);
        if !will_grow {
            self.body.pop_back();
        }
    }
    
    /// Calculate where the snake's head will be after moving forward
    #[inline(always)]
    pub fn calculate_new_head(&self) -> Point {
        let current_head = self.body.get(0).unwrap();
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
    snake: CachePadded<Snake>,
}

impl GridAwareSnake {
    /// Create a new GridAwareSnake. The snake will be added to the grid immediately.
    pub fn new(snake: Snake, grid: &mut Grid) -> Self {
        let wrapper = Self { snake: CachePadded::new(snake) };
        
        // Add initial snake body to grid
        wrapper.update_grid_with_body(grid);
        
        wrapper
    }
    
    /// Calculate new head position (no grid access)
    #[inline(always)]
    pub fn calculate_new_head(&self) -> Point {
        self.snake.calculate_new_head()
    }
    
    /// Get current tail position (no grid access)
    #[inline(always)]
    pub fn tail_position(&self) -> Option<Point> {
        if self.snake.body.len() > 0 {
            self.snake.body.get(self.snake.body.len() - 1).copied()
        } else {
            None
        }
    }
    
    /// Update snake body after successful movement (no grid access)
    #[inline(always)]
    pub fn update_body(&mut self, will_grow: bool) {
        self.snake.move_forward(will_grow);
    }
    
    /// Mark snake as dead (no grid access)
    #[inline(always)]
    pub fn mark_dead(&mut self) {
        self.snake.is_alive = false;
    }
    
    /// Move the snake forward, automatically updating the grid
    /// Returns true if movement was successful, false if collision occurred
    #[deprecated(note = "Use cache-aware methods: calculate_new_head(), update_body(), mark_dead()")]
    #[inline(always)]
    pub fn move_forward(&mut self, grid: &mut Grid, will_grow: bool) -> bool {
        // Calculate new head position
        let new_head = self.snake.calculate_new_head();
        
        // Check for collisions BEFORE moving
        if grid.get_cell(&new_head) == Cell::Snake {
            // Collision detected - mark as dead
            self.snake.is_alive = false;
            return false;
        }
        
        // No collision - proceed with movement
        // Only clear the tail position from grid if not growing
        if !will_grow {
            if let Some(tail) = self.snake.body.get(self.snake.body.len() - 1) {
                grid.set_cell(*tail, Cell::Empty);
            }
        }
        
        // Move the snake with growth flag
        self.snake.move_forward(will_grow);
        
        // Update grid with new head position
        if let Some(head) = self.snake.body.get(0) {
            grid.set_cell(*head, Cell::Snake);
        }
        
        return true;
    }
    
    
    /// Mark the snake as dead and clear it from the grid
    #[inline(always)]
    pub fn die(&mut self, grid: &mut Grid) {
        self.snake.is_alive = false;
        self.clear_from_grid(grid);
    }
    
    /// Change direction (no grid update needed)
    #[inline(always)]
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
    #[inline(always)]
    pub fn head(&self) -> Option<&Point> {
        self.snake.body.get(0)
    }
    

    
    /// Get snake body
    pub fn body(&self) -> &TinyDeque<[Point; 16]> {
        &self.snake.body
    }
    
    // Private helper methods

    fn update_grid_with_body(&self, grid: &mut Grid) {
        for i in 0..self.snake.body.len() {
            if let Some(part) = self.snake.body.get(i) {
                grid.set_cell(*part, Cell::Snake);
            }
        }
    }
    
    fn clear_from_grid(&self, grid: &mut Grid) {
        for i in 0..self.snake.body.len() {
            if let Some(part) = self.snake.body.get(i) {
                grid.set_cell(*part, Cell::Empty);
            }
        }
    }
}
