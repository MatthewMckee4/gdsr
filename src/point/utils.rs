use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySequence};

use crate::point::Point;

pub fn py_list_to_points(points: &Bound<'_, PyList>) -> PyResult<Vec<Point>> {
    points
        .iter()
        .map(|point: pyo3::Bound<'_, PyAny>| py_any_to_point(&point))
        .collect()
}

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

pub fn py_sequence_to_py_list<'a>(
    points: &'a Bound<'a, PySequence>,
) -> PyResult<Bound<'a, PyList>> {
    let points_list = match points.to_list() {
        Ok(list) => list,
        Err(_) => {
            return Err(PyTypeError::new_err(
                "Invalid points format: cannot convert to list",
            ));
        }
    };

    check_pylist_not_empty(&points_list)?;

    Ok(points_list)
}

const EMPTY_LIST_ERROR: &str = "Points cannot be empty";

pub fn check_pylist_not_empty(list: &Bound<'_, PyList>) -> PyResult<()> {
    if list.is_empty() {
        Err(PyTypeError::new_err(EMPTY_LIST_ERROR))
    } else {
        Ok(())
    }
}

pub fn check_vec_not_empty(vec: &[Point]) -> PyResult<()> {
    if vec.is_empty() {
        Err(PyTypeError::new_err(EMPTY_LIST_ERROR))
    } else {
        Ok(())
    }
}

pub fn point_str(point: &Point) -> String {
    format!("Point({}, {})", point.x, point.y)
}

pub fn point_repr(point: &Point) -> String {
    format!("({}, {})", point.x, point.y)
}
