use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PyTuple};
use std::collections::HashMap;

pub fn to_points(points: &Bound<'_, PyList>) -> PyResult<Vec<(f64, f64)>> {
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
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Invalid point format",
        ));
    }

    Ok(vec)
}
