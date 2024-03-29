use crate::{
    fireball::FireballObject,
    point::{AsPoint, Point},
    Direction, Object,
};

#[derive(PartialEq, Eq, Debug)]
pub enum Spell {
    Fireball,
}

pub trait Magic {
    fn cost(&self) -> u32;
    fn cooldown(&self) -> u128;
    fn evoke(&self, location: Point, direction: Direction) -> Box<dyn Object>;
    fn get_spell(&self) -> Spell;
}

#[derive(Debug)]
pub struct FireballMagic;

impl FireballMagic {
    pub fn new() -> Self {
        Self {}
    }
}

impl Magic for FireballMagic {
    fn cost(&self) -> u32 {
        10
    }

    fn cooldown(&self) -> u128 {
        4
    }

    fn evoke(&self, location: Point, direction: Direction) -> Box<dyn Object> {
        Box::new(FireballObject::new(
            location + direction.as_point(),
            direction,
        ))
    }

    fn get_spell(&self) -> Spell {
        Spell::Fireball
    }
}
