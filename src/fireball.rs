use crate::{point::Point, Direction};


#[derive(Debug)]
pub struct Fireball {
    pub location: Point,
    pub direction: Direction,
    pub speed: f64,
}
impl Fireball {
    pub fn new(location: Point, direction: Direction) -> Fireball {
        Fireball {
            location,
            direction,
            speed: 0.8,
        }
    }
}