use rand::random;

use super::*;
use crate::{
    point::{AsPoint, Point},
    Coord, Direction,
};

pub struct Monster {
    pub location: Point,
    pub logic: usize,
    pub id: usize,
    pub speed: f64,
}

impl Monster {
    pub fn new_simple(coord: Coord) -> Self {
        Self::new(coord, None, None)
    }

    pub fn new(coord: Coord, logic: Option<usize>, speed: Option<f64>) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            logic: logic.unwrap_or(100),
            speed: speed.unwrap_or(0.8),
            id: random(),
        }
    }
    pub fn seek(&self, seek_point: Point, _elapsed: u128) -> Point {
        let step = match rand::random::<usize>() % self.logic {
            ..=39 => seek_point - self.location,
            ..=59 => Point::new(seek_point.x - self.location.x, 0.0),
            ..=79 => Point::new(0.0, seek_point.y - self.location.y),
            ..=84 => Direction::Right.as_point(),
            ..=89 => Direction::Left.as_point(),
            ..=94 => Direction::Up.as_point(),
            ..=99 => Direction::Down.as_point(),
            _ => Point::new(0.0, 0.0),
        }
        .normalize(self.speed);

        self.location + step
    }
}

impl Unit for Monster {
    fn step(&mut self, step: Point) {
        self.location = step;
    }

    fn speed(&self) -> f64 {
        self.speed
    }
}

impl Entity for Monster {
    fn location(&self) -> Point {
        self.location
    }
}
