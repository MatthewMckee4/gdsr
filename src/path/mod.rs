use path_type::PathType;
use pyo3::prelude::*;

use crate::{
    point::Point,
    traits::{Movable, Rotatable, Scalable},
};

mod general;
mod io;
pub mod path_type;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct Path {
    #[pyo3(get)]
    points: Vec<Point>,
    #[pyo3(get)]
    layer: i32,
    #[pyo3(get)]
    data_type: i32,
    #[pyo3(get, set)]
    path_type: Option<PathType>,
    #[pyo3(get, set)]
    width: Option<f64>,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Path")
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Pa")
    }
}

impl Movable for Path {
    fn move_by(&mut self, delta: Point) -> &mut Self {
        for point in &mut self.points {
            *point += delta;
        }
        self
    }

    fn move_to(&mut self, target: Point) -> &mut Self {
        let delta = target - self.points[0];
        self.move_by(delta)
    }
}

impl Rotatable for Path {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.rotate(angle, centre);
        }
        self
    }
}

impl Scalable for Path {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.scale(factor, centre);
        }
        self
    }
}
