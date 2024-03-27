use crossterm::style::Color;

use crate::Coord;

pub enum Action {
    Move { symbol: char, color: Color, old: Coord, new: Coord },
}
