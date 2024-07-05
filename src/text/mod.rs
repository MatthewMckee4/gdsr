use crate::element::Element;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Text {}

#[pymethods]
impl Text {
    #[new]
    pub fn new() -> Self {
        Text {}
    }
}

impl Element for Text {
    fn __str__(&self) -> PyResult<String> {
        Ok("Text".to_string())
    }
}
