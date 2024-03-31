pub mod monster;
pub mod object;
pub mod player;

use crate::point::Point;

pub trait Unit {
    fn step(&mut self, step: Point);
    fn speed(&self) -> f64;
}

pub trait Entity {
    fn location(&self) -> Point;
}
