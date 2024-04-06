use nalgebra::{vector, Vector2};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub trait AsVector2 {
    fn as_vector(&self) -> Vector2<f64>;
}

impl AsVector2 for Direction {
    fn as_vector(&self) -> Vector2<f64> {
        match self {
            Direction::Up => vector![0.0, -1.0],
            Direction::Down => vector![0.0, 1.0],
            Direction::Left => vector![-1.0, 0.0],
            Direction::Right => vector![1.0, 0.0],
        }
    }
}
