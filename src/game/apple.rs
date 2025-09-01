use crate::game::grid::{Cell, Grid};
use crate::game::types::Point;

pub const APPLE_CAPACITY: usize = 128;

pub struct Apple {
    pub position: Point,
}

impl Apple {
    pub fn new(position: Point) -> Self {
        Self { position }
    }
}

/// Smart wrapper around Apple that automatically manages grid updates
pub struct GridAwareApple {
    apple: Apple,
    is_spawned: bool,
}

impl GridAwareApple {
    /// Create a new GridAwareApple. The apple will be added to the grid immediately.
    pub fn new(apple: Apple, grid: &mut Grid) -> Self {
        let mut wrapper = Self {
            apple,
            is_spawned: true,
        };
        
        // Add apple to grid
        grid.set_cell(wrapper.apple.position, Cell::Apple);
        
        wrapper
    }
    
    /// Create a new GridAwareApple without spawning it to the grid
    pub fn new_unspawned(apple: Apple, _grid: &mut Grid) -> Self {
        Self {
            apple,
            is_spawned: false,
        }
    }
    
    /// Spawn the apple to the grid
    #[inline(always)]
    pub fn spawn(&mut self, grid: &mut Grid) {
        if !self.is_spawned {
            grid.set_cell(self.apple.position, Cell::Apple);
            self.is_spawned = true;
        }
    }
    
    /// Consume the apple, removing it from the grid
    #[inline(always)]
    pub fn consume(&mut self, grid: &mut Grid) {
        if self.is_spawned {
            grid.set_cell(self.apple.position, Cell::Empty);
            self.is_spawned = false;
        }
    }
    
    /// Move the apple to a new position, updating the grid
    pub fn move_to(&mut self, new_position: Point, grid: &mut Grid) {
        // Clear old position if spawned
        if self.is_spawned {
            grid.set_cell(self.apple.position, Cell::Empty);
        }
        
        // Update position
        self.apple.position = new_position;
        
        // Add to new position if spawned
        if self.is_spawned {
            grid.set_cell(self.apple.position, Cell::Apple);
        }
    }
    
    /// Get a reference to the underlying apple
    pub fn apple(&self) -> &Apple {
        &self.apple
    }
    
    /// Get apple position
    pub fn position(&self) -> Point {
        self.apple.position
    }
    
    /// Check if apple is spawned on the grid
    pub fn is_spawned(&self) -> bool {
        self.is_spawned
    }
    
}
