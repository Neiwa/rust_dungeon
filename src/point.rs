use std::ops;

use crate::Direction;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn normalize(&self) -> Point {
        if self.x == 0.0 && self.y == 0.0 {
            return *self;
        }
        let fac = 1.0 / (self.x.powi(2) + self.y.powi(2)).sqrt();
        Point::new(self.x * fac, self.y * fac)
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign<Point> for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Mul<i32> for Point {
    type Output = Point;

    fn mul(self, rhs: i32) -> Self::Output {
        Point {
            x: self.x * f64::from(rhs),
            y: self.y * f64::from(rhs),
        }
    }
}

impl Direction {
    pub fn as_point(self) -> Point {
        match self {
            Self::Up => Point::new(0.0, -1.0),
            Self::Down => Point::new(0.0, 1.0),
            Self::Left => Point::new(-1.0, 0.0),
            Self::Right => Point::new(1.0, 0.0),
        }
    }
}
