use pyo3::prelude::*;

use super::Library;
use crate::cell::Cell;

#[pymethods]
impl Library {
    #[new]
    pub fn new(name: String) -> Self {
        Library {
            name,
            cells: Vec::new(),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Library {} with {} cells",
            self.name,
            self.cells.len()
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.name.clone())
    }

    #[pyo3(signature=(*cells))]
    pub fn add(
        &mut self,
        #[pyo3(from_py_with = "input_cells_to_correct_format")] cells: Vec<Cell>,
    ) -> PyResult<()> {
        for cell in cells {
            self.cells.push(cell);
        }
        Ok(())
    }

    #[pyo3(signature=(*cells))]
    pub fn remove(
        &mut self,
        #[pyo3(from_py_with = "input_cells_to_correct_format")] cells: Vec<Cell>,
    ) -> PyResult<()> {
        for cell in cells {
            self.cells.retain(|x| x != &cell);
        }
        Ok(())
    }
}
