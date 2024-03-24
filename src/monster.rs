use rand::random;

use crate::{point::Point, Coord, Direction};

pub struct Player {
    pub coord: Coord,
}

pub struct Monster {
    pub location: Point,
    pub coord: Coord,
    pub last_coord: Coord,
    pub speed: i32,
    pub id: i32,
}

impl Monster {
    pub fn new(coord: Coord) -> Self {
        Self::new_complex(coord, None)
    }

    pub fn new_complex(coord: Coord, speed: Option<i32>) -> Self {
        Self {
            location: Point::new(coord.x as f64, coord.y as f64),
            coord: coord,
            last_coord: coord,
            speed: speed.unwrap_or(100),
            id: random(),
        }
    }

    fn step(&mut self, step: Coord) -> Coord {
        self.last_coord = self.coord;
        self.coord += step;
        self.coord
    }

    pub fn seek(&mut self, seek_coord: Coord) -> Coord {
        let step = match rand::random::<i32>() % self.speed {
            ..=19 => {
                if seek_coord.x - self.coord.x == 0 {
                    Coord::new(0, (seek_coord.y - self.coord.y).clamp(-1, 1))
                } else {
                    Coord::new((seek_coord.x - self.coord.x).clamp(-1, 1), 0)
                }
            }
            ..=39 => {
                if seek_coord.y - self.coord.y == 0 {
                    Coord::new((seek_coord.x - self.coord.x).clamp(-1, 1), 0)
                } else {
                    Coord::new(0, (seek_coord.y - self.coord.y).clamp(-1, 1))
                }
            }
            ..=59 => Coord::new((seek_coord.x - self.coord.x).clamp(-1, 1), 0),
            ..=79 => Coord::new(0, (seek_coord.y - self.coord.y).clamp(-1, 1)),
            ..=84 => Direction::Right.as_coord(),
            ..=89 => Direction::Left.as_coord(),
            ..=94 => Direction::Up.as_coord(),
            ..=99 => Direction::Down.as_coord(),
            _ => Coord::new(0, 0),
        };

        self.step(step)
    }
}
