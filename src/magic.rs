use crate::{point::Point, Direction, Object};

pub mod fireball;

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
