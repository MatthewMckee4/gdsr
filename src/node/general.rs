use pyo3::prelude::*;

use super::Node;

#[pymethods]
impl Node {
    #[new]
    pub fn new() -> Self {
        Node {}
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
