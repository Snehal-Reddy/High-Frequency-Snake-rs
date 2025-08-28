#[cfg(test)]
mod tests {
    use crate::game::{
        engine::GameState,
        snake::Snake,
        apple::Apple,
        types::{Direction, Point},
        grid::{Grid, Cell},
    };

    // Basic Functional Tests
    #[test]
    fn test_snake_movement() {
        let mut snake = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        let initial_head = *snake.body.front().unwrap();
        
        snake.move_forward();
        let new_head = *snake.body.front().unwrap();
        
        assert_eq!(new_head.x, initial_head.x + 1);
        assert_eq!(new_head.y, initial_head.y);
    }

    #[test]
    fn test_snake_boundary_wrapping() {
        let mut snake = Snake::new(1, Point { x: 999, y: 500 }, Direction::Right);
        snake.move_forward();
        let new_head = *snake.body.front().unwrap();
        
        assert_eq!(new_head.x, 0); // Should wrap to 0
        assert_eq!(new_head.y, 500);
    }

    #[test]
    fn test_snake_growth() {
        let mut snake = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        let initial_length = snake.body.len();
        
        snake.grow();
        assert_eq!(snake.body.len(), initial_length + 1);
    }

    #[test]
    fn test_snake_direction_change() {
        let mut snake = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        
        // Should be able to change direction
        snake.change_direction(Direction::Up);
        assert_eq!(snake.direction, Direction::Up);
        
        // Should not be able to reverse direction
        snake.change_direction(Direction::Down);
        assert_eq!(snake.direction, Direction::Up); // Should remain Up
    }

    // Integration Tests
    #[test]
    fn test_apple_consumption() {
        let mut game = GameState::new();
        
        // Add a snake
        let snake = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        game.snakes.insert(1, snake);
        
        // Add an apple at the snake's next position (after movement)
        let apple = Apple::new(Point { x: 501, y: 500 });
        game.apples.push(apple);
        
        let initial_apple_count = game.apples.len();
        let initial_snake_length = game.snakes.get(&1).unwrap().body.len();
        
        game.tick(&[]);
        
        // Snake should have grown and apple should be consumed (but a new one spawned)
        assert_eq!(game.apples.len(), initial_apple_count); // Count stays the same
        assert_eq!(game.snakes.get(&1).unwrap().body.len(), initial_snake_length + 1);
    }

    #[test]
    fn test_snake_collision() {
        let mut game = GameState::new();
        
        // Add two snakes that will collide on the next move
        let snake1 = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        let snake2 = Snake::new(2, Point { x: 502, y: 500 }, Direction::Left);
        
        game.snakes.insert(1, snake1);
        game.snakes.insert(2, snake2);
        
        let initial_snake_count = game.snakes.len();
        
        game.tick(&[]);
        
        // Both snakes should move to position 501 and collide
        assert!(game.snakes.len() < initial_snake_count);
    }

    #[test]
    fn test_input_processing() {
        let mut game = GameState::new();
        
        // Add a snake
        let snake = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        game.snakes.insert(1, snake);
        
        // Create input to change direction
        let input = crate::game::types::Input {
            snake_id: 1,
            direction: Direction::Up,
        };
        
        game.tick(&[input]);
        
        // Snake should have changed direction
        assert_eq!(game.snakes.get(&1).unwrap().direction, Direction::Up);
    }

    #[test]
    fn test_dead_snake_cleanup() {
        let mut game = GameState::new();
        
        // Add a snake
        let mut snake = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        snake.is_alive = false; // Mark as dead
        game.snakes.insert(1, snake);
        
        let initial_snake_count = game.snakes.len();
        
        game.tick(&[]);
        
        // Dead snake should be removed
        assert_eq!(game.snakes.len(), initial_snake_count - 1);
    }

    #[test]
    fn test_game_state_consistency() {
        let game = GameState::random();
        
        // Verify all snakes are within grid bounds
        for snake in game.snakes.values() {
            for part in &snake.body {
                assert!(part.x < 1000);
                assert!(part.y < 1000);
            }
        }
        
        // Verify all apples are within grid bounds
        for apple in &game.apples {
            assert!(apple.position.x < 1000);
            assert!(apple.position.y < 1000);
        }
        
        // Verify grid consistency
        for snake in game.snakes.values() {
            for part in &snake.body {
                assert_eq!(game.grid.get_cell(part), Cell::Snake);
            }
        }
        
        for apple in &game.apples {
            assert_eq!(game.grid.get_cell(&apple.position), Cell::Apple);
        }
    }

    #[test]
    fn test_multiple_ticks() {
        let mut game = GameState::new();
        
        // Add a snake
        let snake = Snake::new(1, Point { x: 500, y: 500 }, Direction::Right);
        game.snakes.insert(1, snake);
        
        // Add an apple
        let apple = Apple::new(Point { x: 501, y: 500 });
        game.apples.push(apple);
        game.grid.set_cell(Point { x: 501, y: 500 }, Cell::Apple);
        game.grid.set_cell(Point { x: 500, y: 500 }, Cell::Snake);
        
        // Run multiple ticks
        for _ in 0..5 {
            game.tick(&[]);
        }
        
        // Game should still be in a valid state
        assert!(game.snakes.len() > 0);
        assert!(game.snakes.get(&1).unwrap().is_alive);
    }

    #[test]
    fn test_grid_operations() {
        let mut grid = Grid::new();
        let point = Point { x: 100, y: 200 };
        
        // Test setting and getting cells
        assert_eq!(grid.get_cell(&point), Cell::Empty);
        
        grid.set_cell(point, Cell::Snake);
        assert_eq!(grid.get_cell(&point), Cell::Snake);
        
        grid.set_cell(point, Cell::Apple);
        assert_eq!(grid.get_cell(&point), Cell::Apple);
    }
}
