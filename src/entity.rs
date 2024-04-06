pub mod monster;
pub mod object;
pub mod player;

use nalgebra::Point2;

pub trait Unit {
    fn set_location(&mut self, location: Point2<f64>, ticker: u128);
    fn speed(&self) -> f64;
}

pub trait Entity {
    fn location(&self) -> Point2<f64>;
}
