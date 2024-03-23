pub mod console_monster;

use crate::{coord::Coord, Direction};

pub trait Unit {
    fn coord(&self) -> Coord;
    fn set_coord(&mut self, coord: Coord) -> Coord;

    fn step(&mut self, step: Coord) -> Coord {
        self.set_coord(self.coord() + step)
    }
}

pub struct Player<TRepresentation> {
    pub coord: Coord,
    pub repr: TRepresentation,
}

impl<TRepresentation> Unit for Player<TRepresentation> {
    fn coord(&self) -> Coord {
        self.coord
    }

    fn set_coord(&mut self, coord: Coord) -> Coord {
        self.coord = coord;
        self.coord
    }
}

pub struct Monster<TRepresentation> {
    pub coord: Coord,
    pub last_coord: Coord,
    pub speed: i32,
    pub repr: TRepresentation,
}

impl<TRepresentation> Monster<TRepresentation> {
    // pub fn seek(&mut self, seek_coord: Coord) -> Coord {
    //     let step = match rand::random::<i32>() % self.speed {
    //         ..=19 => {
    //             if seek_coord.x - self.coord.x == 0 {
    //                 Coord::new(0, (seek_coord.y - self.coord.y).clamp(-1, 1))
    //             } else {
    //                 Coord::new((seek_coord.x - self.coord.x).clamp(-1, 1), 0)
    //             }
    //         }
    //         ..=39 => {
    //             if seek_coord.y - self.coord.y == 0 {
    //                 Coord::new((seek_coord.x - self.coord.x).clamp(-1, 1), 0)
    //             } else {
    //                 Coord::new(0, (seek_coord.y - self.coord.y).clamp(-1, 1))
    //             }
    //         }
    //         ..=59 => Coord::new((seek_coord.x - self.coord.x).clamp(-1, 1), 0),
    //         ..=79 => Coord::new(0, (seek_coord.y - self.coord.y).clamp(-1, 1)),
    //         ..=84 => Direction::Right.as_coord(),
    //         ..=89 => Direction::Left.as_coord(),
    //         ..=94 => Direction::Up.as_coord(),
    //         ..=99 => Direction::Down.as_coord(),
    //         _ => Coord::new(0, 0),
    //     };

    //     self.step(step)
    // }
}
