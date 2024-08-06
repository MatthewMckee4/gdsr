use std::collections::HashMap;

use pyo3::prelude::*;

use crate::cell::Cell;

use super::Library;

#[pymethods]
impl Library {
    #[new]
    #[pyo3(signature = (name=String::from("Library")))]
    pub fn new(name: String) -> Self {
        Library {
            name,
            cells: HashMap::new(),
        }
    }

    #[pyo3(signature = (*cells))]
    pub fn add(&mut self, cells: Vec<Py<Cell>>, py: Python) -> PyResult<()> {
        for cell in cells {
            self.cells
                .insert(cell.borrow(py).name.clone(), cell.clone_ref(py));
        }
        Ok(())
    }
    #[pyo3(signature = (*cells))]
    pub fn remove(&mut self, cells: Vec<Py<Cell>>, py: Python) -> PyResult<()> {
        for cell in cells {
            self.cells.remove(&cell.borrow(py).name);
        }
        Ok(())
    }

    fn __str__(&self) -> PyResult<String> {
        let cells_len = self.cells.len();
        Ok(format!("Library {} with {} cells", self.name, cells_len))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Library({:?})", self.name))
    }
}
