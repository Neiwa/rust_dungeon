use crate::{
    magic::{Magic, Spell},
    point::Point,
    Object,
};

#[derive(Debug)]
pub struct FireballObject {
    pub location: Point,
    pub vector: Point,
    pub speed: f64,
    last_tick: u128,
}

impl FireballObject {
    pub fn new(location: Point, vector: Point, ticker: u128) -> Self {
        Self {
            location,
            vector: vector.normalize(0.01),
            speed: 0.01,
            last_tick: ticker,
        }
    }
}

impl Object for FireballObject {
    fn location(&self) -> Point {
        self.location
    }

    fn vector(&self) -> Point {
        self.vector
    }

    fn speed(&self) -> f64 {
        self.speed
    }

    fn set_location(&mut self, location: Point, ticker: u128) {
        self.location = location;
        self.last_tick = ticker;
    }

    fn get_spell(&self) -> Spell {
        Spell::Fireball
    }

    fn next_location(&self, ticker: u128) -> Point {
        self.location + self.vector * (ticker - self.last_tick)
    }
}

#[derive(Debug)]
pub struct FireballMagic {
    last_evoke: Option<u128>,
}

impl FireballMagic {
    pub fn new() -> Self {
        Self { last_evoke: None }
    }
}

impl Magic for FireballMagic {
    fn cost(&self) -> u32 {
        10
    }

    fn cooldown(&self) -> u128 {
        4
    }

    fn evoke(&mut self, location: Point, direction: Point, ticker: u128) -> Vec<Box<dyn Object>> {
        self.last_evoke = Some(ticker);

        vec![Box::new(FireballObject::new(
            location + direction.normalize(1.0),
            direction,
            ticker,
        ))]
    }

    fn get_spell(&self) -> Spell {
        Spell::Fireball
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
