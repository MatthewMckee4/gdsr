use path_type::PathType;
use pyo3::prelude::*;

use crate::point::Point;

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
