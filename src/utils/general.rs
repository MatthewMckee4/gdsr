use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use crate::point::Point;

pub fn check_points_vec_not_empty(vec: &[Point]) -> PyResult<()> {
    if vec.is_empty() {
        Err(PyTypeError::new_err("Points cannot be empty"))
    } else {
        Ok(())
    }
}
