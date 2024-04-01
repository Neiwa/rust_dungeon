use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};

use super::*;
use crate::{
    point::{AsPoint, Point},
    Coord, Direction,
};

pub struct Monster {
    pub location: Point,
    pub logic: usize,
    pub id: u64,
    pub speed: f64,
    last_tick: u128,
}

impl Monster {
    pub fn new_simple(coord: Coord, ticker: u128) -> Self {
        Self::new(coord, ticker, None, None)
    }

    pub fn new(coord: Coord, ticker: u128, logic: Option<usize>, speed: Option<f64>) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            logic: logic.unwrap_or(100),
            speed: speed.unwrap_or(0.002),
            id: random(),
            last_tick: ticker,
        }
    }
    pub fn seek(&self, seek_point: Point, ticker: u128) -> Point {
        let mut rng = StdRng::seed_from_u64((ticker as u64 / 2000).wrapping_add(self.id));

        let step = match rng.gen::<usize>() % self.logic {
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

        self.location + step * ticker.saturating_sub(self.last_tick)
    }
}

impl Unit for Monster {
    fn speed(&self) -> f64 {
        self.speed
    }

    fn set_location(&mut self, location: Point, ticker: u128) {
        self.location = location;
        self.last_tick = ticker;
    }
}

impl Entity for Monster {
    fn location(&self) -> Point {
        self.location
    }
}
