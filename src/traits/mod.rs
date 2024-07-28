use pyo3::prelude::*;
use std::fs::File;

use crate::point::Point;

pub trait ToGds {
    fn _to_gds(&self, file: File, scale: f64) -> PyResult<File>;
}

pub trait Movable {
    fn move_to(&mut self, point: Point);
    fn move_by(&mut self, vector: Point);
}
