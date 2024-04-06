use nalgebra::{Point2, Vector2};

use crate::{magic::Spell, Entity};

pub trait Object {
    fn location(&self) -> Point2<f64>;
    fn vector(&self) -> Vector2<f64>;
    fn set_location(&mut self, location: Point2<f64>, ticker: u128);
    fn get_spell(&self) -> Spell;
    fn next_location(&self, ticker: u128) -> Point2<f64>;
}

impl<T: Object> Entity for T {
    fn location(&self) -> Point2<f64> {
        self.location()
    }
}
