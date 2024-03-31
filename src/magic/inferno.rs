use crate::{
    magic::fireball::FireballObject,
    point::{AsPoint, Point},
    Direction, Object,
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
        200
    }

    fn evoke(&mut self, location: Point, _direction: Point, ticker: u128) -> Vec<Box<dyn Object>> {
        self.last_evoke = Some(ticker);

        vec![
            Box::new(FireballObject::new(
                location + Direction::Up.as_point() + Direction::Left.as_point(),
                Direction::Up.as_point() + Direction::Left.as_point() * 2,
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point() + Direction::Left.as_point(),
                Direction::Up.as_point() + Direction::Left.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point() + Direction::Left.as_point(),
                Direction::Up.as_point() * 2 + Direction::Left.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point(),
                Direction::Up.as_point() * 2 + Direction::Left.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point(),
                Direction::Up.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point(),
                Direction::Up.as_point() * 2 + Direction::Right.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point() + Direction::Right.as_point(),
                Direction::Up.as_point() * 2 + Direction::Right.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point() + Direction::Right.as_point(),
                Direction::Up.as_point() + Direction::Right.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Up.as_point() + Direction::Right.as_point(),
                Direction::Up.as_point() + Direction::Right.as_point() * 2,
            )),
            Box::new(FireballObject::new(
                location + Direction::Right.as_point(),
                Direction::Right.as_point() * 2 + Direction::Up.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Right.as_point(),
                Direction::Right.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Right.as_point(),
                Direction::Right.as_point() * 2 + Direction::Down.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point() + Direction::Right.as_point(),
                Direction::Down.as_point() + Direction::Right.as_point() * 2,
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point() + Direction::Right.as_point(),
                Direction::Down.as_point() + Direction::Right.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point() + Direction::Right.as_point(),
                Direction::Down.as_point() * 2 + Direction::Right.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point(),
                Direction::Down.as_point() * 2 + Direction::Right.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point(),
                Direction::Down.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point(),
                Direction::Down.as_point() * 2 + Direction::Left.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point() + Direction::Left.as_point(),
                Direction::Down.as_point() * 2 + Direction::Left.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point() + Direction::Left.as_point(),
                Direction::Down.as_point() + Direction::Left.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Down.as_point() + Direction::Left.as_point(),
                Direction::Down.as_point() + Direction::Left.as_point() * 2,
            )),
            Box::new(FireballObject::new(
                location + Direction::Left.as_point(),
                Direction::Left.as_point() * 2 + Direction::Down.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Left.as_point(),
                Direction::Left.as_point(),
            )),
            Box::new(FireballObject::new(
                location + Direction::Left.as_point(),
                Direction::Left.as_point() * 2 + Direction::Up.as_point(),
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
