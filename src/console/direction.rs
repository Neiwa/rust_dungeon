use std::ops;

use crate::{AsCoord, Coord};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl AsCoord for Direction {
    fn as_coord(&self) -> Coord {
        match self {
            Self::Up => Coord::new(0, -1),
            Self::Down => Coord::new(0, 1),
            Self::Left => Coord::new(-1, 0),
            Self::Right => Coord::new(1, 0),
        }
    }
}

pub trait AsDirection {
    fn as_direction(self) -> Option<Direction>;
}

impl ops::Add<Direction> for Coord {
    type Output = Coord;

    fn add(self, rhs: Direction) -> Self::Output {
        self + rhs.as_coord()
    }
}

impl ops::AddAssign<Direction> for Coord {
    fn add_assign(&mut self, rhs: Direction) {
        let coord = rhs.as_coord();
        self.x += coord.x;
        self.y += coord.y;
    }
}
