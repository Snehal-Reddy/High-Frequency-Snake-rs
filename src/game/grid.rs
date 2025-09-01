use crate::game::types::Point;

pub const GRID_WIDTH: usize = 4000;
pub const GRID_HEIGHT: usize = 4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Snake,
    Apple,
}

pub struct Grid {
    cells: Vec<Vec<Cell>>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: vec![vec![Cell::Empty; GRID_WIDTH]; GRID_HEIGHT],
        }
    }

    #[inline(always)]
    pub fn get_cell(&self, point: &Point) -> Cell {
        self.cells[point.y as usize][point.x as usize]
    }

    #[inline(always)]
    pub fn set_cell(&mut self, point: Point, cell: Cell) {
        self.cells[point.y as usize][point.x as usize] = cell;
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}
