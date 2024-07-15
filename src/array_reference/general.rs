use pyo3::prelude::*;

use crate::cell::Cell;

use super::ArrayReference;

#[pymethods]
impl ArrayReference {
    #[new]
    pub fn new(cell: Cell) -> Self {
        ArrayReference { cell }
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
