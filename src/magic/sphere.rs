use crate::{
    magic::{Magic, Spell},
    point::{AsPoint, Point},
    Direction, Object,
};

#[derive(Debug)]
pub struct SphereObject {
    pub location: Point,
    pub direction: Direction,
    pub speed: f64,
}

impl SphereObject {
    pub fn new(location: Point, direction: Direction) -> Self {
        Self {
            location,
            direction,
            speed: 0.5,
        }
    }
}

impl Object for SphereObject {
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
        Spell::Sphere
    }
}

#[derive(Debug)]
pub struct SphereMagic;

impl SphereMagic {
    pub fn new() -> Self {
        Self {}
    }
}

impl Magic for SphereMagic {
    fn cost(&self) -> u32 {
        5
    }

    fn cooldown(&self) -> u128 {
        2
    }

    fn evoke(&self, location: Point, direction: Direction) -> Box<dyn Object> {
        match direction {
            Direction::Up | Direction::Down => Box::new(SphereObject::new(
                location + direction.as_point(),
                direction,
            )),
            Direction::Left | Direction::Right => Box::new(SphereObject::new(
                location + direction.as_point() * 2,
                direction,
            )),
        }
    }

    fn get_spell(&self) -> Spell {
        Spell::Sphere
    }
}
