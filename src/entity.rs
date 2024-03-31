pub mod monster;
pub mod object;
pub mod player;

use crate::point::Point;

pub trait UnitLogic {
    fn step(&mut self, step: Point);
    fn speed(&self) -> f64;
}
