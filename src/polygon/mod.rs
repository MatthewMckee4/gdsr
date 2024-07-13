use std::{
    fmt,
    hash::{Hash, Hasher},
};

use crate::point::Point;
use pyo3::prelude::*;
use utils::{polygon_repr, polygon_str};

mod general;
mod utils;

#[pyclass(get_all)]
#[derive(Clone)]
pub struct Polygon {
    points: Vec<Point>,
    layer: i32,
    data_type: i32,
}

impl Hash for Polygon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.points.hash(state);
        self.layer.hash(state);
        self.data_type.hash(state);
    }
}

impl PartialEq for Polygon {
    fn eq(&self, other: &Self) -> bool {
        self.points == other.points
            && self.layer == other.layer
            && self.data_type == other.data_type
    }
}

impl PartialOrd for Polygon {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.layer == other.layer {
            if self.data_type == other.data_type {
                return self.points.partial_cmp(&other.points);
            }
            return self.data_type.partial_cmp(&other.data_type);
        }
        self.layer.partial_cmp(&other.layer)
    }
}

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", polygon_str(self))
    }
}

impl fmt::Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", polygon_repr(self))
    }
}
