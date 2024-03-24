pub mod coord;

use crossterm::style::Color;

use crate::{Coord, Monster};

pub trait ConsoleUnit {
    fn color(&self) -> Color;
    fn symbol(&self) -> char;
    fn coord(&self) -> Coord;
}

impl ConsoleUnit for Monster {
    fn color(&self) -> Color {
        match self.id % 11 {
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
        }
    }

    fn symbol(&self) -> char {
        match self.id % 5 {
            0 => 'W',
            1 => 'O',
            2 => 'X',
            3 => 'F',
            _ => 'C',
        }
    }

    fn coord(&self) -> Coord {
        self.coord
    }
}