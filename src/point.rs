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

    pub fn normalize(&self, magnitude: f64) -> Point {
        if self.x == 0.0 && self.y == 0.0 {
            return *self;
        }
        let len = (self.x.powi(2) + self.y.powi(2)).sqrt();
        let fac = magnitude / len;
        *self * fac
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

impl ops::Add<Option<Point>> for Point {
    type Output = Self;

    fn add(self, rhs: Option<Point>) -> Self::Output {
        if let Some(r) = rhs {
            return self + r;
        }
        self
    }
}

impl ops::Add<Point> for Option<Point> {
    type Output = Self;

    fn add(self, rhs: Point) -> Self::Output {
        if let Some(lhs) = self {
            return Some(lhs + rhs);
        }
        Some(rhs)
    }
}

impl ops::AddAssign<Point> for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::AddAssign<Option<Point>> for Point {
    fn add_assign(&mut self, rhs: Option<Point>) {
        if let Some(r) = rhs {
            self.x += r.x;
            self.y += r.y;
        }
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

impl ops::Mul<f64> for Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Sub<Point> for Point {
    type Output = Self;

    fn sub(self, rhs: Point) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ops::Sub<Option<Point>> for Point {
    type Output = Self;

    fn sub(self, rhs: Option<Point>) -> Self::Output {
        if let Some(r) = rhs {
            return self - r;
        }
        self
    }
}

pub trait AsPoint {
    fn as_point(&self) -> Point;
}

impl AsPoint for Direction {
    fn as_point(&self) -> Point {
        match self {
            Self::Up => Point::new(0.0, -1.0),
            Self::Down => Point::new(0.0, 1.0),
            Self::Left => Point::new(-1.0, 0.0),
            Self::Right => Point::new(1.0, 0.0),
        }
    }
}
