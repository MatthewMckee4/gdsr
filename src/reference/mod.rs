use crate::element::Element;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Reference {}

#[pymethods]
impl Reference {
    #[new]
    pub fn new() -> Self {
        Reference {}
    }
}

impl Element for Reference {
    fn __str__(&self) -> PyResult<String> {
        Ok("Reference".to_string())
    }
}
