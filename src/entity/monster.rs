use nalgebra::{vector, Point2};
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};

use crate::console::{AsVector2, Direction};

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
        let id = random();
        Self {
            location,
            logic: logic.unwrap_or(100),
            speed: speed.unwrap_or(2.) / 1000.,
            id,
            last_tick: ticker,
        }
    }

    pub fn set_ticker(&mut self, ticker: u128) {
        self.last_tick = ticker;
    }

    pub fn seek(&self, seek_point: Point2<f64>, ticker: u128) -> Option<Point2<f64>> {
        let mut rng = StdRng::seed_from_u64((ticker as u64).wrapping_add(self.id) / 2000);

        let step = match rng.gen::<usize>() % self.logic {
            ..=39 => Some(seek_point - self.location),
            ..=59 => Some(vector!(seek_point.x - self.location.x, 0.)),
            ..=79 => Some(vector!(0., seek_point.y - self.location.y)),
            ..=84 => Some(Direction::Right.as_vector()),
            ..=89 => Some(Direction::Left.as_vector()),
            ..=94 => Some(Direction::Up.as_vector()),
            ..=99 => Some(Direction::Down.as_vector()),
            _ => None,
        };

        if step.is_none() {
            return None;
        }

        Some(
            self.location
                + step.unwrap().normalize()
                    * self.speed
                    * ticker.saturating_sub(self.last_tick) as f64,
        )
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
