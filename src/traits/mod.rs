use pyo3::prelude::*;
use std::fs::File;

use crate::point::Point;

pub trait ToGds {
    fn _to_gds(&self, file: File, scale: f64) -> PyResult<File>;
}

pub trait Movable {
    fn move_to(&mut self, point: Point) -> &mut Self;
    fn move_by(&mut self, vector: Point) -> &mut Self;
}

pub trait Rotatable {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self;
}

pub trait Scalable {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self;
}
