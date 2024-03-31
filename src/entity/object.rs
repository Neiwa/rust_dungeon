use crate::{magic::Spell, point::Point};

pub trait Object {
    fn location(&self) -> Point;
    fn vector(&self) -> Point;
    fn speed(&self) -> f64;
    fn set_location(&mut self, location: Point);
    fn get_spell(&self) -> Spell;
}
