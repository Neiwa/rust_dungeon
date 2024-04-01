pub mod monster;
pub mod object;
pub mod player;

use crate::point::Point;

pub trait Unit {
    fn set_location(&mut self, location: Point, ticker: u128);
    fn speed(&self) -> f64;
}

pub trait Entity {
    fn location(&self) -> Point;
}
