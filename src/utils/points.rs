use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PySequence, PyTuple};
use std::collections::HashMap;

pub fn to_points(points: Bound<'_, PyList>) -> PyResult<Vec<(f64, f64)>> {
    let mut vec: Vec<(f64, f64)> = Vec::new();

    for point in points.iter() {
        if let Ok(tuple) = point.downcast::<PyTuple>() {
            if tuple.len() == 2 {
                if let (Ok(x), Ok(y)) = (
                    tuple.get_item(0)?.extract::<f64>(),
                    tuple.get_item(1)?.extract::<f64>(),
                ) {
                    vec.push((x, y));
                    continue;
                }
            }
        }
        if let Ok(seq) = point.extract::<Vec<f64>>() {
            if seq.len() == 2 {
                vec.push((seq[0], seq[1]));
                continue;
            }
        }
        if let Ok(mapping) = point.extract::<HashMap<i32, f64>>() {
            if let (Some(x), Some(y)) = (mapping.get(&0), mapping.get(&1)) {
                vec.push((*x, *y));
                continue;
            }
        }
        if let Ok(indexable) = point.downcast::<PyAny>() {
            if let (Ok(x), Ok(y)) = (
                indexable.get_item(0)?.extract::<f64>(),
                indexable.get_item(1)?.extract::<f64>(),
            ) {
                vec.push((x, y));
                continue;
            }
        }
        return Err(PyTypeError::new_err("Invalid point format"));
    }

    Ok(vec)
}

pub fn to_required_input_points_format<'a>(
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

pub fn check_vec_not_empty(vec: &Vec<(f64, f64)>) -> PyResult<()> {
    if vec.is_empty() {
        Err(PyTypeError::new_err(EMPTY_LIST_ERROR))
    } else {
        Ok(())
    }
}
