use rand::random;

use crate::{point::Point, Coord, Direction};

pub struct Player {
    pub location: Point,
    pub last_coord: Coord,
}

impl Player {
    pub fn new(coord: Coord) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            last_coord: coord,
        }
    }
}

pub struct Unit {
    pub location: Point,
    pub last_coord: Coord,
    pub speed: i32,
    pub id: i32,
}

pub trait UnitLogic {
    fn step(&mut self, step: Point);
}

pub trait Monster {
    fn seek(&self, seek_point: Point, _elapsed: u128) -> Point;
}

impl Monster for Unit {
    fn seek(&self, seek_point: Point, _elapsed: u128) -> Point {
        let step = match rand::random::<i32>() % self.speed {
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
        .normalize_max(1.0);

        self.location + step
    }
}

impl UnitLogic for Unit {
    fn step(&mut self, step: Point) {
        self.last_coord = self.location.as_coord();
        self.location = step;
    }
}

impl UnitLogic for Player {
    fn step(&mut self, step: Point) {
        self.last_coord = self.location.as_coord();
        self.location = step;
    }
}

impl Unit {
    pub fn new_simple(coord: Coord) -> Self {
        Self::new(coord, None)
    }

    pub fn new(coord: Coord, speed: Option<i32>) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            last_coord: coord,
            speed: speed.unwrap_or(100),
            id: random(),
        }
    }
}
