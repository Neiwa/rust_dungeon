use crossterm::style::Color;
use nalgebra::Point2;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RenderAction {
    Move {
        symbol: char,
        color: Color,
        old: Point2<f64>,
        new: Point2<f64>,
    },
    Remove {
        coord: Point2<f64>,
        symbol: char,
    },
    Create {
        symbol: char,
        color: Color,
        location: Point2<f64>,
    },
}
