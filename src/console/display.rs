use std::collections::HashMap;

use crossterm::style::Color;

use crate::point::Point;

use super::Coord;

pub struct Display<'a> {
    pub status_indicators: HashMap<&'a str, Indicator>,
    top_left: Coord,
    bottom_right: Coord,
    resolution: Point,
}

impl<'a> Display<'a> {
    pub fn new(top_left: Coord, bottom_right: Coord, resolution: Point) -> Self {
        let top_right = Coord::new(bottom_right.x, top_left.y);
        let bottom_left = Coord::new(top_left.x, bottom_right.y);

        Self {
            top_left,
            bottom_right,
            resolution,
            status_indicators: HashMap::from([
                ("clock", Indicator::new(top_right + Coord::new(-6, 0))),
                ("score", Indicator::new(top_left + Coord::new(4, 0))),
                ("spells", Indicator::new(bottom_left + Coord::new(4, 0))),
                ("energy", Indicator::new(bottom_right + Coord::new(-9, 0))),
            ]),
        }
    }
}

pub struct Indicator {
    pub coord: Coord,
    pub color: Color,
    pub bg_color: Color,
}

impl Indicator {
    fn new(coord: Coord) -> Self {
        Self {
            coord,
            color: Color::White,
            bg_color: Color::Magenta,
        }
    }
}
