use rand::random;

use crate::magic::fireball::FireballMagic;
use crate::magic::sphere::SphereMagic;
use crate::{
    magic::{Magic, Spell},
    point::{AsPoint, Point},
    Coord, Direction,
};

pub struct Player {
    pub location: Point,
    pub energy: u32,
    pub max_energy: u32,
    pub last_shot: u128,
    pub magic: Vec<Box<dyn Magic>>,
    pub active_spell: usize,
}

impl Player {
    pub fn new(coord: Coord) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            energy: 100,
            max_energy: 100,
            last_shot: 0,
            magic: vec![Box::new(FireballMagic::new()), Box::new(SphereMagic::new())],
            active_spell: 1,
        }
    }

    pub fn get_active_spell(&self) -> &Box<dyn Magic> {
        &self.magic[self.active_spell]
    }
}

pub struct Unit {
    pub location: Point,
    pub logic: usize,
    pub id: usize,
    pub speed: f64,
}

pub trait Object {
    fn location(&self) -> Point;
    fn direction(&self) -> Direction;
    fn speed(&self) -> f64;
    fn set_location(&mut self, location: Point);
    fn get_spell(&self) -> Spell;
}

pub trait UnitLogic {
    fn step(&mut self, step: Point);
    fn speed(&self) -> f64;
}

pub trait Monster {
    fn seek(&self, seek_point: Point, _elapsed: u128) -> Point;
}

impl Monster for Unit {
    fn seek(&self, seek_point: Point, _elapsed: u128) -> Point {
        let step = match rand::random::<usize>() % self.logic {
            ..=39 => Point::new(
                seek_point.x - self.location.x,
                seek_point.y - self.location.y,
            ),
            ..=59 => Point::new(seek_point.x - self.location.x, 0.0),
            ..=79 => Point::new(0.0, seek_point.y - self.location.y),
            ..=84 => Direction::Right.as_point(),
            ..=89 => Direction::Left.as_point(),
            ..=94 => Direction::Up.as_point(),
            ..=99 => Direction::Down.as_point(),
            _ => Point::new(0.0, 0.0),
        }
        .normalize(self.speed);

        self.location + step
    }
}

impl UnitLogic for Unit {
    fn step(&mut self, step: Point) {
        self.location = step;
    }

    fn speed(&self) -> f64 {
        self.speed
    }
}

impl UnitLogic for Player {
    fn step(&mut self, step: Point) {
        self.location = step;
    }

    fn speed(&self) -> f64 {
        1.0
    }
}

impl Unit {
    pub fn new_simple(coord: Coord) -> Self {
        Self::new(coord, None, None)
    }

    pub fn new(coord: Coord, logic: Option<usize>, speed: Option<f64>) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            logic: logic.unwrap_or(100),
            speed: speed.unwrap_or(0.8),
            id: random(),
        }
    }
}
