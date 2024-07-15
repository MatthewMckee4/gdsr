use pyo3::prelude::*;

use super::Path;

#[pymethods]
impl Path {
    #[new]
    pub fn new() -> Self {
        Path {}
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
