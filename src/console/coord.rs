use std::ops;

use crate::point::Point;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn as_point(&self) -> Point {
        Point {
            x: self.x as f64,
            y: self.y as f64,
        }
    }
}

impl Point {
    pub fn as_coord(&self) -> Coord {
        Coord {
            x: self.x.round() as i32,
            y: self.y.round() as i32,
        }
    }
}

impl ops::Add<Coord> for Coord {
    type Output = Coord;

    fn add(self, _rhs: Coord) -> Coord {
        Coord {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
        }
    }
}

impl ops::AddAssign<Coord> for Coord {
    fn add_assign(&mut self, rhs: Coord) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    pub fn as_coord(self) -> Coord {
        match self {
            Self::Up => Coord::new(0, -1),
            Self::Down => Coord::new(0, 1),
            Self::Left => Coord::new(-1, 0),
            Self::Right => Coord::new(1, 0),
        }
    }
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