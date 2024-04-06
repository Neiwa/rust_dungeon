use nalgebra::{Point2, Vector2};

use crate::{
    magic::{Magic, Spell},
    Object,
};

#[derive(Debug)]
pub struct SphereObject {
    pub location: Point2<f64>,
    pub vector: Vector2<f64>,
    pub speed: f64,
    last_tick: u128,
}

impl SphereObject {
    pub fn new(location: Point2<f64>, direction: Vector2<f64>, ticker: u128) -> Self {
        Self {
            location,
            vector: direction.normalize() * 0.005,
            speed: 0.005,
            last_tick: ticker,
        }
    }
}

impl Object for SphereObject {
    fn location(&self) -> Point2<f64> {
        self.location
    }

    fn vector(&self) -> Vector2<f64> {
        self.vector
    }

    fn speed(&self) -> f64 {
        self.speed
    }

    fn set_location(&mut self, location: Point2<f64>, ticker: u128) {
        self.location = location;
        self.last_tick = ticker;
    }

    fn get_spell(&self) -> Spell {
        Spell::Sphere
    }

    fn next_location(&self, ticker: u128) -> Point2<f64> {
        self.location + self.vector * ticker.saturating_sub(self.last_tick) as f64
    }
}

#[derive(Debug)]
pub struct SphereMagic {
    last_evoke: Option<u128>,
}

impl SphereMagic {
    pub fn new() -> Self {
        Self { last_evoke: None }
    }
}

impl Magic for SphereMagic {
    fn cost(&self) -> u32 {
        5
    }

    fn cooldown(&self) -> u128 {
        400
    }

    fn evoke(
        &mut self,
        location: Point2<f64>,
        direction: Vector2<f64>,
        ticker: u128,
    ) -> Vec<Box<dyn Object>> {
        self.last_evoke = Some(ticker);

        vec![Box::new(SphereObject::new(
            location + direction.normalize(),
            direction,
            ticker,
        ))]
    }

    fn get_spell(&self) -> Spell {
        Spell::Sphere
    }

    fn on_cooldown(&self, ticker: u128) -> bool {
        self.remaining_cooldown(ticker) > 0
    }

    fn remaining_cooldown(&self, ticker: u128) -> u128 {
        if let Some(last_evoke) = self.last_evoke {
            return (last_evoke + self.cooldown()).saturating_sub(ticker);
        }
        0
    }
}
