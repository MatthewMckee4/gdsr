use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyAny, PySequence, PyTuple},
};

use crate::{cell::Cell, point::Point, utils::io::create_temp_file};

use super::general::check_points_vec_not_empty;

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

#[allow(unused)]
pub fn py_any_to_cells_vec(cells: &Bound<'_, PyAny>) -> PyResult<Vec<Cell>> {
    let mut result = Vec::new();
    if let Ok(cells) = cells.downcast::<PyTuple>() {
        for cell in cells.iter() {
            if let Ok(cell) = cell.extract::<Cell>() {
                result.push(cell);
            } else {
                return Err(PyTypeError::new_err("Invalid cell format: not a Cell"));
            }
        }
    } else {
        return Err(PyTypeError::new_err("Invalid cells format: not a tuple"));
    }
    Ok(result)
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

pub fn py_any_path_to_string_or_temp_name(
    file_name: &Bound<'_, PyAny>,
) -> PyResult<Option<String>> {
    if file_name.is_none() {
        return Ok(None);
    }

    match py_any_path_to_string(file_name) {
        Ok(file_name) => Ok(Some(file_name)),
        Err(_) => match create_temp_file() {
            Ok(temp_file_name) => Ok(Some(temp_file_name)),
            Err(_) => Err(PyTypeError::new_err("Failed to create a temporary file")),
        },
    }
}
pub fn py_any_path_to_string(file_name: &Bound<'_, PyAny>) -> PyResult<String> {
    match file_name.call_method0("__str__") {
        Ok(py_str) => py_str
            .extract()
            .map_err(|_| PyTypeError::new_err("Failed to convert to string")),
        Err(_) => Err(PyTypeError::new_err("Invalid path format")),
    }
}
