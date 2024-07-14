use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::cell::Cell;

pub fn input_cells_to_correct_format(cells: &Bound<'_, PyAny>) -> PyResult<Vec<Cell>> {
    let mut result = Vec::new();
    if let Ok(cells) = cells.downcast::<PyTuple>() {
        for cell in cells.iter() {
            if let Ok(cell) = cell.extract::<Cell>() {
                result.push(cell);
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err("Invalid element"));
            }
        }
    } else {
        return Err(pyo3::exceptions::PyTypeError::new_err("Invalid elements"));
    }
    Ok(result)
}
