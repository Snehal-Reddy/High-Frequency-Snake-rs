use crate::game::types::Point;

pub struct Apple {
    pub position: Point,
}

impl Apple {
    pub fn new(position: Point) -> Self {
        Self { position }
    }
}
