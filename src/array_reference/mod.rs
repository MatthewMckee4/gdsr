use crate::element::Element;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct ArrayReference {}

#[pymethods]
impl ArrayReference {
    #[new]
    pub fn new() -> Self {
        ArrayReference {}
    }
}

impl Element for ArrayReference {
    fn __str__(&self) -> PyResult<String> {
        Ok("ArrayReference".to_string())
    }
}
