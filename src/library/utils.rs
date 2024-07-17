use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::cell::Cell;

#[allow(unused)]
pub fn input_cells_to_correct_format(cells: &Bound<'_, PyAny>) -> PyResult<Vec<Cell>> {
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
