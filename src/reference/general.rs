use pyo3::prelude::*;

use crate::cell::Cell;

use super::Reference;

#[pymethods]
impl Reference {
    #[new]
    pub fn new(cell: Cell) -> Self {
        Reference { cell }
    }

    fn copy(&self) -> PyResult<Self> {
        Ok(self.clone())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
