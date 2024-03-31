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
}

impl FireballObject {
    pub fn new(location: Point, vector: Point) -> Self {
        Self {
            location,
            vector: vector.normalize(0.8),
            speed: 0.8,
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

    fn set_location(&mut self, location: Point) {
        self.location = location
    }

    fn get_spell(&self) -> Spell {
        Spell::Fireball
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
