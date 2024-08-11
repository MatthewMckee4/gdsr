use std::collections::HashMap;

use pyo3::{exceptions::PyValueError, prelude::*};

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

    #[pyo3(signature = (*cells, replace_pre_existing=false))]
    pub fn add(&mut self, cells: Vec<Py<Cell>>, replace_pre_existing: bool) -> PyResult<()> {
        Python::with_gil(|py| {
            for cell in cells {
                if !replace_pre_existing && self.cells.contains_key(&cell.borrow(py).name) {
                    return Err(PyValueError::new_err(format!(
                        "Cell with name {} already exists in library",
                        cell.borrow(py).name
                    )));
                }
                self.cells
                    .insert(cell.borrow(py).name.clone(), cell.clone_ref(py));
            }
            Ok(())
        })
    }

    #[pyo3(signature = (*cells))]
    pub fn remove(&mut self, cells: Vec<Py<Cell>>, py: Python) -> PyResult<()> {
        for cell in cells {
            self.cells.remove(&cell.borrow(py).name);
        }
        Ok(())
    }

    pub fn contains(&self, cell: Py<Cell>, py: Python) -> bool {
        let cell = cell.borrow(py);
        for c in self.cells.values() {
            if c.borrow(py).eq(&cell) {
                return true;
            }
        }
        false
    }

    #[pyo3(signature = (deep=false))]
    pub fn copy(&self, deep: bool, py: Python) -> PyResult<Self> {
        let mut cells: HashMap<String, Py<Cell>> = HashMap::new();
        for (key, value) in &self.cells {
            if deep {
                cells.insert(key.clone(), Py::new(py, value.borrow(py).clone())?);
            } else {
                cells.insert(key.clone(), value.clone_ref(py));
            }
        }
        Ok(Library {
            name: self.name.clone(),
            cells,
        })
    }

    fn __add__(mut slf: PyRefMut<'_, Self>, cell: Py<Cell>) -> PyRefMut<'_, Self> {
        let _ = slf.add([cell].to_vec(), true);
        slf
    }

    pub fn __contains__(&self, cell: Py<Cell>, py: Python) -> bool {
        self.contains(cell, py)
    }

    pub fn __eq__(&self, other: &Self, py: Python) -> bool {
        if (self.name != other.name) || (self.cells.len() != other.cells.len()) {
            return false;
        }
        for (key, value) in &self.cells {
            if !other.cells.contains_key(key)
                || !value
                    .borrow(py)
                    .eq(&other.cells.get(key).unwrap().borrow(py))
            {
                return false;
            }
        }
        true
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
