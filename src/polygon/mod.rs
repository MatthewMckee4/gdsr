use crate::{point::Point, traits::Movable};
use pyo3::prelude::*;

mod general;
mod io;
mod utils;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct Polygon {
    #[pyo3(get)]
    points: Vec<Point>,
    #[pyo3(get)]
    layer: i32,
    #[pyo3(get)]
    data_type: i32,
}

impl std::fmt::Display for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Polygon with {} point(s), starting at ({}, {}) on layer {}, data type {}",
            self.points.len(),
            self.points[0].x,
            self.points[0].y,
            self.layer,
            self.data_type
        )
    }
}

impl std::fmt::Debug for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Po({:?}, n={}, l={}, d={})",
            self.points[0],
            self.points.len(),
            self.layer,
            self.data_type
        )
    }
}

impl Movable for Polygon {
    fn move_by(&mut self, delta: Point) {
        for point in &mut self.points {
            *point += delta;
        }
    }

    fn move_to(&mut self, target: Point) {
        let delta = target - self.points[0];
        self.move_by(delta);
    }
}
