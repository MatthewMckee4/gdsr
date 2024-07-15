use pyo3::prelude::*;

use super::Text;

#[pymethods]
impl Text {
    #[new]
    pub fn new() -> Self {
        Text {}
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
