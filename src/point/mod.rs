use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use pyo3::prelude::*;

pub mod general;
pub mod iterator;
pub mod utils;

pub use iterator::PointIterator;
pub use utils::*;

#[pyclass(frozen, get_all)]
#[derive(Clone, Copy, PartialEq, Default)]

pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.x == other.x {
            return self.y.partial_cmp(&other.y);
        }
        self.x.partial_cmp(&other.x)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Mul<f64> for Point {
    type Output = Self;

    fn mul(self, factor: f64) -> Self::Output {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl Mul<Point> for f64 {
    type Output = Point;

    fn mul(self, point: Point) -> Self::Output {
        Point {
            x: self * point.x,
            y: self * point.y,
        }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        *self = Point {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Div<f64> for Point {
    type Output = Self;

    fn div(self, divisor: f64) -> Self::Output {
        if divisor == 0.0 {
            panic!("Division by zero");
        }
        Point {
            x: self.x / divisor,
            y: self.y / divisor,
        }
    }
}
