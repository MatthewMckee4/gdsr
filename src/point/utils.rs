use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::point::Point;

pub fn py_any_to_point(point: &Bound<'_, PyAny>) -> PyResult<Point> {
    if let (Ok(x), Ok(y)) = (point.get_item(0), point.get_item(1)) {
        match (x.extract::<f64>(), y.extract::<f64>()) {
            (Ok(x), Ok(y)) => Ok(Point::new(x, y)),
            _ => Err(PyTypeError::new_err(
                "Invalid point format: items are not floats",
            )),
        }
    } else {
        Err(PyTypeError::new_err(
            "Invalid point format: item is not indexable",
        ))
    }
}

pub fn get_points_from_i32_vec(vec: Vec<i32>) -> Vec<Point> {
    vec.chunks(2)
        .map(|chunk| Point::new(chunk[0] as f64, chunk[1] as f64))
        .collect()
}
