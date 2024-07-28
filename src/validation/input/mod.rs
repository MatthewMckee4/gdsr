use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::PySequence,
};

use crate::{
    point::{py_any_to_point, Point},
    utils::general::check_points_vec_not_empty,
};

pub fn check_layer_valid(layer: i32) -> PyResult<()> {
    if !(0..=255).contains(&layer) {
        return Err(PyValueError::new_err("Layer must be in the range 0-255"));
    }
    Ok(())
}

pub fn check_data_type_valid(_: i32) -> PyResult<()> {
    Ok(())
}

pub fn py_any_to_points_vec(points: &Bound<'_, PyAny>) -> PyResult<Vec<Point>> {
    if let Ok(points) = points.downcast::<PySequence>() {
        let mut points_list = Vec::new();
        for item in points.iter()? {
            let point = py_any_to_point(&item?)?;
            points_list.push(point);
        }
        check_points_vec_not_empty(&points_list)?;
        Ok(points_list)
    } else {
        Err(PyTypeError::new_err(
            "Invalid points format: not a sequence",
        ))
    }
}
