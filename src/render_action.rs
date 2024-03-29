use crossterm::style::Color;

use crate::Coord;

pub enum RenderAction {
    Move {
        symbol: char,
        color: Color,
        old: Coord,
        new: Coord,
    },
    Remove(Coord),
    Create {
        symbol: char,
        color: Color,
        coord: Coord,
    },
}
