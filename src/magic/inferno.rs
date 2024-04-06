use nalgebra::{Point2, Vector2};

use crate::{
    console::{AsVector2, Direction},
    magic::fireball::FireballObject,
    Object,
};

use super::{Magic, Spell};

pub struct InfernoMagic {
    last_evoke: Option<u128>,
}

impl InfernoMagic {
    pub fn new() -> Self {
        Self { last_evoke: None }
    }
}

impl Magic for InfernoMagic {
    fn cost(&self) -> u32 {
        80
    }

    fn cooldown(&self) -> u128 {
        40_000
    }

    fn evoke(
        &mut self,
        location: Point2<f64>,
        _direction: Vector2<f64>,
        ticker: u128,
    ) -> Vec<Box<dyn Object>> {
        self.last_evoke = Some(ticker);

        vec![
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector() + Direction::Left.as_vector(),
                Direction::Up.as_vector() + Direction::Left.as_vector() * 2.0,
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector() + Direction::Left.as_vector(),
                Direction::Up.as_vector() + Direction::Left.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector() + Direction::Left.as_vector(),
                Direction::Up.as_vector() * 2.0 + Direction::Left.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector(),
                Direction::Up.as_vector() * 2.0 + Direction::Left.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector(),
                Direction::Up.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector(),
                Direction::Up.as_vector() * 2.0 + Direction::Right.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector() + Direction::Right.as_vector(),
                Direction::Up.as_vector() * 2.0 + Direction::Right.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector() + Direction::Right.as_vector(),
                Direction::Up.as_vector() + Direction::Right.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_vector() + Direction::Right.as_vector(),
                Direction::Up.as_vector() + Direction::Right.as_vector() * 2.0,
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Right.as_vector(),
                Direction::Right.as_vector() * 2.0 + Direction::Up.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Right.as_vector(),
                Direction::Right.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Right.as_vector(),
                Direction::Right.as_vector() * 2.0 + Direction::Down.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector() + Direction::Right.as_vector(),
                Direction::Down.as_vector() + Direction::Right.as_vector() * 2.0,
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector() + Direction::Right.as_vector(),
                Direction::Down.as_vector() + Direction::Right.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector() + Direction::Right.as_vector(),
                Direction::Down.as_vector() * 2.0 + Direction::Right.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector(),
                Direction::Down.as_vector() * 2.0 + Direction::Right.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector(),
                Direction::Down.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector(),
                Direction::Down.as_vector() * 2.0 + Direction::Left.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector() + Direction::Left.as_vector(),
                Direction::Down.as_vector() * 2.0 + Direction::Left.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector() + Direction::Left.as_vector(),
                Direction::Down.as_vector() + Direction::Left.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_vector() + Direction::Left.as_vector(),
                Direction::Down.as_vector() + Direction::Left.as_vector() * 2.0,
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Left.as_vector(),
                Direction::Left.as_vector() * 2.0 + Direction::Down.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Left.as_vector(),
                Direction::Left.as_vector(),
                ticker,
            )),
            Box::new(FireballObject::new(
                location + Direction::Left.as_vector(),
                Direction::Left.as_vector() * 2.0 + Direction::Up.as_vector(),
                ticker,
            )),
        ]
    }

    fn get_spell(&self) -> Spell {
        Spell::Inferno
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
