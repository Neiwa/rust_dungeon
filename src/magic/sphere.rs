use crate::{
    magic::{Magic, Spell},
    point::{AsPoint, Point},
    Direction, Object,
};

#[derive(Debug)]
pub struct SphereObject {
    pub location: Point,
    pub vector: Point,
    pub speed: f64,
}

impl SphereObject {
    pub fn new(location: Point, direction: Direction) -> Self {
        Self {
            location,
            vector: direction.as_point().normalize(0.5),
            speed: 0.5,
        }
    }
}

impl Object for SphereObject {
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

    fn get_spell(&self) -> crate::magic::Spell {
        Spell::Sphere
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
        2
    }

    fn evoke(
        &mut self,
        location: Point,
        direction: Direction,
        ticker: u128,
    ) -> Vec<Box<dyn Object>> {
        self.last_evoke = Some(ticker);

        vec![Box::new(SphereObject::new(
            location + direction.as_point(),
            direction,
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
