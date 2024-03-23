use crossterm::style::Color;

use crate::{Coord, Direction, Monster, Unit};


#[derive(Clone, Copy, Debug)]
pub struct ConsoleRepresentation {
    pub symbol: char,
    pub color: Color,
}

impl ConsoleRepresentation {
    pub fn new(symbol: char, color: Color) -> Self {
        Self { symbol, color }
    }
}

impl Unit for Monster<ConsoleRepresentation> {
    fn coord(&self) -> Coord {
        self.coord
    }

    fn set_coord(&mut self, coord: Coord) -> Coord {
        self.last_coord = self.coord;
        self.coord = coord;
        self.coord
    }
    
    fn step(&mut self, step: Coord) -> Coord {
        self.last_coord = self.coord;
        self.coord += step;
        self.coord
    }
}

impl Monster<ConsoleRepresentation> {
    pub fn new(coord: Coord) -> Self {
        Self::new_console(coord, None, None)
    }
    pub fn new_console(coord: Coord, speed: Option<i32>, symbol: Option<char>) -> Self {
        let color = match rand::random::<u8>() % 11 {
            0 => Color::Grey,
            1 => Color::DarkGrey,
            2 => Color::Red,
            3 => Color::DarkRed,
            4 => Color::Green,
            5 => Color::DarkGreen,
            6 => Color::Yellow,
            7 => Color::DarkYellow,
            8 => Color::Blue,
            9 => Color::DarkBlue,
            10 => Color::DarkMagenta,
            _ => Color::DarkCyan,
        };

        Self {
            coord: coord,
            last_coord: coord,
            speed: speed.unwrap_or(100),
            repr: ConsoleRepresentation::new(symbol.unwrap_or('W'), color),
        }
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