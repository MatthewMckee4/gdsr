use pyo3::prelude::*;

use super::Box;

#[pymethods]
impl Box {
    #[new]
    pub fn new() -> Self {
        Box {}
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
