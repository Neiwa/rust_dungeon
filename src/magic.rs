use crate::{object::Object, point::Point};

pub mod fireball;
pub mod inferno;
pub mod sphere;

#[derive(PartialEq, Eq, Debug)]
pub enum Spell {
    Fireball,
    Sphere,
    Inferno,
}

pub trait Magic {
    fn cost(&self) -> u32;
    fn cooldown(&self) -> u128;
    fn remaining_cooldown(&self, ticker: u128) -> u128;
    fn on_cooldown(&self, ticker: u128) -> bool;
    fn evoke(&mut self, location: Point, direction: Point, ticker: u128) -> Vec<Box<dyn Object>>;
    fn get_spell(&self) -> Spell;
}
