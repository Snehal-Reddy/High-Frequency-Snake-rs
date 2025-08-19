use crate::game::types::{Direction, Point};
use std::collections::VecDeque;

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
                y: head.y.wrapping_sub(1),
            },
            Direction::Down => Point {
                x: head.x,
                y: head.y.wrapping_add(1),
            },
            Direction::Left => Point {
                x: head.x.wrapping_sub(1),
                y: head.y,
            },
            Direction::Right => Point {
                x: head.x.wrapping_add(1),
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
