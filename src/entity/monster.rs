use nalgebra::{vector, Point2};
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};

use super::*;

pub struct Monster {
    pub location: Point2<f64>,
    pub logic: usize,
    pub id: u64,
    pub speed: f64,
    last_tick: u128,
}

impl Monster {
    pub fn new_simple(location: Point2<f64>, ticker: u128) -> Self {
        Self::new(location, ticker, None, None)
    }

    pub fn new(
        location: Point2<f64>,
        ticker: u128,
        logic: Option<usize>,
        speed: Option<f64>,
    ) -> Self {
        Self {
            location,
            logic: logic.unwrap_or(100),
            speed: speed.unwrap_or(2.0) / 1000.0,
            id: random(),
            last_tick: ticker,
        }
    }
    pub fn seek(&self, seek_point: Point2<f64>, ticker: u128) -> Point2<f64> {
        let mut rng = StdRng::seed_from_u64((ticker as u64).wrapping_add(self.id) / 2000);

        let step = match rng.gen::<usize>() % self.logic {
            ..=39 => seek_point - self.location,
            ..=59 => vector!(seek_point.x - self.location.x, 0.0),
            ..=79 => vector!(0.0, seek_point.y - self.location.y),
            ..=84 => vector!(1.0, 0.0),
            ..=89 => vector!(-1.0, 0.0),
            ..=94 => vector!(0.0, -1.0),
            ..=99 => vector!(0.0, 1.0),
            _ => vector!(0.0, 0.0),
        }
        .normalize()
            * self.speed;

        self.location + step * ticker.saturating_sub(self.last_tick) as f64
    }
}

impl Unit for Monster {
    fn speed(&self) -> f64 {
        self.speed
    }

    fn set_location(&mut self, location: Point2<f64>, ticker: u128) {
        self.location = location;
        self.last_tick = ticker;
    }
}

impl Entity for Monster {
    fn location(&self) -> Point2<f64> {
        self.location
    }
}
