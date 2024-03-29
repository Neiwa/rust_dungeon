use crate::{magic::Spell, point::Point, Direction, Object};

#[derive(Debug)]
pub struct FireballObject {
    pub location: Point,
    pub direction: Direction,
    pub speed: f64,
}

impl FireballObject {
    pub fn new(location: Point, direction: Direction) -> FireballObject {
        FireballObject {
            location,
            direction,
            speed: 0.8,
        }
    }
}

impl Object for FireballObject {
    fn location(&self) -> Point {
        self.location
    }

    fn direction(&self) -> Direction {
        self.direction
    }

    fn speed(&self) -> f64 {
        self.speed
    }

    fn set_location(&mut self, location: Point) {
        self.location = location
    }

    fn get_spell(&self) -> crate::magic::Spell {
        Spell::Fireball
    }
}
