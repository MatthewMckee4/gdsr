use log::info;
use pyo3::{exceptions::PyValueError, prelude::*, types::PySequence};

use crate::point::{py_list_to_points, py_sequence_to_py_list, Point};

use super::Polygon;

pub fn check_layer_valid(layer: i32) -> PyResult<()> {
    if !(0..=255).contains(&layer) {
        return Err(PyValueError::new_err("Layer must be in the range 0-255"));
    }
    Ok(())
}

pub fn check_data_type_valid(_: i32) -> PyResult<()> {
    Ok(())
}

pub fn input_polygon_points_to_correct_format(points: &Bound<'_, PyAny>) -> PyResult<Vec<Point>> {
    if let Ok(points) = points.downcast::<PySequence>() {
        let points_vec = py_list_to_points(&py_sequence_to_py_list(points)?)?;
        Ok(polygon_points_to_correct_format(points_vec))
    } else {
        Err(PyValueError::new_err(
            "Invalid points format: not a sequence",
        ))
    }
}

fn are_points_closed(points: &[Point]) -> bool {
    points.first() == points.last()
}

pub fn close_points(points: &[Point]) -> Vec<Point> {
    if are_points_closed(points) {
        points.to_vec()
    } else {
        info!("The points {:?} are not closed, closing them", points);
        let mut closed_points = points.to_vec();
        closed_points.push(points[0]);
        closed_points
    }
}

pub fn polygon_points_to_correct_format(points: Vec<Point>) -> Vec<Point> {
    close_points(&points)
}

pub fn polygon_str(polygon: &Polygon) -> String {
    format!(
        "Polygon with {} point(s), starting at ({}, {}) on layer {}, data type {}",
        polygon.points.len(),
        polygon.points[0].x,
        polygon.points[0].y,
        polygon.layer,
        polygon.data_type
    )
}

pub fn polygon_repr(polygon: &Polygon) -> String {
    format!(
        "P({:?}, n={}, l={}, d={})",
        polygon.points[0],
        polygon.points.len(),
        polygon.layer,
        polygon.data_type
    )
}
