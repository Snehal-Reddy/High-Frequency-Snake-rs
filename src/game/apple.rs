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
