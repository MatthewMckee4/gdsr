use pyo3::{exceptions::PyValueError, prelude::*};

use crate::point::Point;

pub fn check_layer_valid(layer: i32) -> PyResult<()> {
    if !(0..=255).contains(&layer) {
        return Err(PyValueError::new_err("Layer must be in the range 0-255"));
    }
    Ok(())
}

pub fn check_data_type_valid(_: i32) -> PyResult<()> {
    Ok(())
}

pub fn check_points_vec_has_at_least_two_points(points: &[Point]) -> PyResult<()> {
    if points.len() < 2 {
        return Err(PyValueError::new_err("Path must have at least two points"));
    }
    Ok(())
}
