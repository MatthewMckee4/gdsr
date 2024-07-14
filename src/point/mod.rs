use std::{
    fmt,
    hash::{Hash, Hasher},
};

use pyo3::prelude::*;

pub mod general;
pub mod iterator;
pub mod utils;

pub use iterator::PointIterator;
pub use utils::*;

#[pyclass(frozen, get_all)]
#[derive(Clone, Copy)]

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

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
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
        write!(f, "{}", point_str(self))
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", point_repr(self))
    }
}
