use crate::game::types::{Direction, Point};
use crate::game::grid::{GRID_WIDTH, GRID_HEIGHT};
use std::{collections::VecDeque};

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
        let head = self.body.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => Point {
                x: head.x,
                y: if head.y == 0 { (GRID_HEIGHT - 1) as u16 } else { head.y - 1 },
            },
            Direction::Down => Point {
                x: head.x,
                y: if head.y == (GRID_HEIGHT - 1) as u16 { 0 } else { head.y + 1 },
            },
            Direction::Left => Point {
                x: if head.x == 0 { (GRID_WIDTH - 1) as u16 } else { head.x - 1 },
                y: head.y,
            },
            Direction::Right => Point {
                x: if head.x == (GRID_WIDTH - 1) as u16 { 0 } else { head.x + 1 },
                y: head.y,
            },
        };
        self.body.push_front(new_head);
        self.body.pop_back();
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
