use rand::distr::{Distribution, StandardUniform};
use rand::Rng;
use crate::game::grid::{GRID_HEIGHT, GRID_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Distribution<Point> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point {
        Point {
            // TODO: replace 1000 with pub var
            x: rng.random_range(0..GRID_WIDTH) as u16,
            y: rng.random_range(0..GRID_HEIGHT) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.random_range(0..4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub snake_id: u32,
    pub direction: Direction,
}
