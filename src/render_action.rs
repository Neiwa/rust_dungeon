use crossterm::style::Color;

use crate::Coord;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RenderAction {
    Move {
        symbol: char,
        color: Color,
        old: Coord,
        new: Coord,
    },
    Remove {
        coord: Coord,
        symbol: char,
    },
    Create {
        symbol: char,
        color: Color,
        coord: Coord,
    },
}
