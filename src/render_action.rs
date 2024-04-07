use crossterm::style::Color;
use nalgebra::Point2;

use crate::{console::ConsoleUnit, Entity};

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

#[derive(Clone, Copy)]
pub enum RenderAction2<'a> {
    Move {
        unit: &'a Box<dyn ConsoleUnit>,
        old: Point2<f64>,
        new: Point2<f64>,
    },
    Remove {
        coord: Point2<f64>,
        unit: &'a Box<dyn ConsoleUnit>,
    },
    Create {
        unit: &'a Box<dyn ConsoleUnit>,
        location: Point2<f64>,
    },
}
