use crate::element::Element;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Path {}

#[pymethods]
impl Path {
    #[new]
    pub fn new() -> Self {
        Path {}
    }
}

impl Element for Path {
    fn __str__(&self) -> PyResult<String> {
        Ok("Path".to_string())
    }
}
